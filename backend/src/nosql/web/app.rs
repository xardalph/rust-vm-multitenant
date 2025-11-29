use super::super::users::Backend as usersBackend;
use super::super::web::middleware::agent_token_validation::{
    self, check_api_token_against_agent_table,
};
use crate::nosql::users;
use crate::nosql::web::controller::auth;
use crate::nosql::web::controller::{protected, public, victoria_api};
use axum::Json;
use serde::{Deserialize, Serialize};

use axum::response::IntoResponse;
use axum::{extract::FromRef, middleware};
use axum_login::tracing::info;
use axum_login::{AuthManagerLayer, AuthManagerLayerBuilder, login_required};
use axum_messages::MessagesManagerLayer;

use clap::Parser;
use sqlx::{Pool as sqlxPool, any::install_default_drivers, migrate::MigrateDatabase};
use std::clone;
use std::sync::Arc;
use time::Duration;
use tokio::{signal, task::AbortHandle};
use tower_sessions::cookie::Key;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_redis_store::{RedisStore, fred::prelude::*};
#[derive(Parser, Debug, Clone)]
struct Opts {
    /// Database connexion string (with username, password, ip and port).
    #[arg(
        short = 'd',
        long = "db",
        env = "DB_URL",
        default_value = "postgres://postgres:password@127.0.0.1:5432/postgres"
    )]
    db_url: String,
    /// redis cache connexion string (with username, password, ip and port).
    #[arg(
        short = 'r',
        long = "redis",
        env = "REDIS_URL",
        default_value = "redis://localhost:6379"
    )]
    redis_url: String,
    /// victoria metric connexion string (with username, password, ip and port).
    #[arg(
        short = 'v',
        long = "victoria",
        env = "VM_URL",
        default_value = "http://VMuser:password@localhost:8427"
    )]
    victoria_metric: String,
}

#[derive(Debug, Clone)]
pub struct App {
    db: sqlxPool<sqlx::Postgres>,
    http: reqwest::Client,
    redis: Pool,
    victoria_metric_url: VictoriaEndpoint,
}
#[derive(Debug, Clone)]
pub struct VictoriaEndpoint {
    pub url: String,
}
// this allow to retrieve each tool from the main App struct in each controller without taking the whole object each time.
impl FromRef<App> for sqlxPool<sqlx::Postgres> {
    fn from_ref(app_state: &App) -> sqlxPool<sqlx::Postgres> {
        app_state.db.clone()
    }
}
impl FromRef<App> for reqwest::Client {
    fn from_ref(app_state: &App) -> reqwest::Client {
        app_state.http.clone()
    }
}
impl FromRef<App> for VictoriaEndpoint {
    fn from_ref(app_state: &App) -> VictoriaEndpoint {
        app_state.victoria_metric_url.clone()
    }
}

impl App {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let opt = Opts::parse();
        info!("starting server with config : {:#?}", opt);

        install_default_drivers();

        let db = sqlx::PgPool::connect(&opt.db_url).await?;
        info!("connection to DB successful, applying migrations...");
        sqlx::migrate!().run(&db).await?;

        let http_client = reqwest::Client::builder().build()?;

        info!("Starting redis configuration...");
        let config = Config::from_url(&opt.redis_url).expect("cannot create config from url");
        let redis_pool =
            Pool::new(config, None, None, None, 6).expect("could not create Redis pool");
        let redis_conn = redis_pool.connect();

        redis_pool.wait_for_connect().await?;
        info!("redis is up.");

        Ok(Self {
            db: db,
            http: http_client,
            redis: redis_pool,
            victoria_metric_url: VictoriaEndpoint {
                url: opt.victoria_metric,
            },
        })
    }

    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        let app = protected::router()
            .merge(auth::router())
            .merge(victoria_api::router().layer(middleware::from_fn_with_state(
                self.db.clone(),
                check_api_token_against_agent_table,
            )))
            .merge(public::router())
            .layer(MessagesManagerLayer)
            .layer(get_auth_layer(self.db.clone(), self.redis.clone()).await)
            .with_state(self.clone());

        info!("listening to port 3000");
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

        // Ensure we use a shutdown signal to abort the deletion task.
        axum::serve(listener, app.into_make_service())
            //.with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle()))
            .await?;

        //deletion_task.await??;

        Ok(())
    }
}

pub async fn get_auth_layer(
    db: sqlxPool<sqlx::Postgres>,
    redis: Pool,
) -> AuthManagerLayer<users::Backend, RedisStore<tower_sessions_redis_store::fred::clients::Pool>> {
    // Session layer.
    //
    // This uses `tower-sessions` to establish a layer that will provide the session
    // as a request extension.

    let session_store = RedisStore::new(redis);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)))
        .with_secure(false);

    // Auth service.
    //
    // This combines the session layer with our backend to establish the auth
    // service which will provide the auth session as a request extension.
    let user_backend = usersBackend::new(db);
    return AuthManagerLayerBuilder::new(user_backend, session_layer.clone()).build();
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
