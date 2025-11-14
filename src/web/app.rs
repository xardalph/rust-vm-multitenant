use std::sync::Arc;

use crate::{
    users::Backend,
    web::{auth, protected, public},
};
use axum_login::{login_required, AuthManagerLayerBuilder};
use axum_messages::MessagesManagerLayer;
use sqlx::{any::install_default_drivers, migrate::MigrateDatabase, Pool as sqlxPool};
use time::Duration;
use tokio::{signal, task::AbortHandle};
use tower_sessions::cookie::Key;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};
pub struct App {
    db: sqlxPool<sqlx::Any>,
}

impl App {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("new");
        let db_url = "sqlite:///tmp/rust-nosql-sqlite.sql";
        install_default_drivers();
        if !sqlx::Sqlite::database_exists(&db_url).await? {
            sqlx::Sqlite::create_database(&db_url).await?;
        }
        let db = sqlx::AnyPool::connect(db_url).await?;
        sqlx::migrate!().run(&db).await?;
        println!("new ended");
        Ok(Self { db: db })
    }

    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        // Session layer.
        //
        // This uses `tower-sessions` to establish a layer that will provide the session
        // as a request extension.
        println!("Starting redis configuration...");
        let redis_url = "redis://127.0.0.1:6379".to_string();
        println!("doing a redis pool...");
        let config = Config::from_url(&redis_url).expect("cannot create config from url");
        println!("made an config from url");
        let redis_pool =
            Pool::new(config, None, None, None, 6).expect("could not create Redis pool");
        println!("got a redis pool");
        let redis_conn = redis_pool.connect();
        println!("connected");
        redis_pool.wait_for_connect().await?;

        let session_store = RedisStore::new(redis_pool);
        let session_layer = SessionManagerLayer::new(session_store)
            .with_expiry(Expiry::OnInactivity(Duration::days(1)))
            .with_secure(false);

        // Auth service.
        //
        // This combines the session layer with our backend to establish the auth
        // service which will provide the auth session as a request extension.
        let backend = Backend::new(self.db.clone());
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();
        let app = protected::router()
            .route_layer(login_required!(Backend, login_url = "/login"))
            .merge(auth::router())
            .merge(public::router())
            .layer(MessagesManagerLayer)
            .layer(auth_layer)
            // TODO: cloning here to have one db con for the auth lib and one for myself, maybe there is a way to have only one ?
            .with_state(self.db.clone());

        println!("listening to port 3000");
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

        // Ensure we use a shutdown signal to abort the deletion task.
        axum::serve(listener, app.into_make_service())
            //.with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle()))
            .await?;

        // deletion_task.await??;

        Ok(())
    }
}

async fn shutdown_signal(deletion_task_abort_handle: AbortHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { deletion_task_abort_handle.abort() },
        _ = terminate => { deletion_task_abort_handle.abort() },
    }
}
