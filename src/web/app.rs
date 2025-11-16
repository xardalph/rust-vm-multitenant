use std::sync::Arc;

use crate::{
    users::Backend as usersBackend,
    web::{
        auth,
        middleware::agent_token_validation::{self, check_api_token_against_agent_table},
        protected, public, victoria_api,
    },
};
use axum::{extract::FromRef, middleware};
use axum_login::{AuthManagerLayerBuilder, login_required};
use axum_messages::MessagesManagerLayer;
use sqlx::{Pool as sqlxPool, any::install_default_drivers, migrate::MigrateDatabase};
use time::Duration;
use tokio::{signal, task::AbortHandle};
use tower_sessions::cookie::Key;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_redis_store::{RedisStore, fred::prelude::*};

#[derive(Debug, Clone)]
pub struct App {
    db: sqlxPool<sqlx::Any>,
    http: reqwest::Client,
}
// this allow to retrieve each tool from the main App struct in each controller without taking the whole object each time.
impl FromRef<App> for sqlxPool<sqlx::Any> {
    fn from_ref(app_state: &App) -> sqlxPool<sqlx::Any> {
        app_state.db.clone()
    }
}
impl FromRef<App> for reqwest::Client {
    fn from_ref(app_state: &App) -> reqwest::Client {
        app_state.http.clone()
    }
}
impl App {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("new");
        //let db_url = "sqlite:";
        let db_url = "postgres://postgres:password@127.0.0.1:5432/postgres";
        install_default_drivers();
        //if !sqlx::Sqlite::database_exists(&db_url).await? {
        //    sqlx::Sqlite::create_database(&db_url).await?;
        //}
        let db = sqlx::AnyPool::connect(db_url).await?;
        sqlx::migrate!().run(&db).await?;
        let http_client = reqwest::Client::builder().build()?;
        Ok(Self {
            db: db,
            http: http_client,
        })
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
        let user_backend = usersBackend::new(self.db.clone());
        let user_auth_layer =
            AuthManagerLayerBuilder::new(user_backend, session_layer.clone()).build();
        let app = protected::router()
            .route_layer(login_required!(usersBackend, login_url = "/login"))
            .merge(auth::router())
            .merge(public::router())
            .merge(victoria_api::router().layer(middleware::from_fn_with_state(
                self.db.clone(),
                check_api_token_against_agent_table,
            )))
            .layer(MessagesManagerLayer)
            .layer(user_auth_layer)
            .with_state(self.clone());

        println!("listening to port 3000");
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

        // Ensure we use a shutdown signal to abort the deletion task.
        axum::serve(listener, app.into_make_service())
            //.with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle()))
            .await?;

        //deletion_task.await??;

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
