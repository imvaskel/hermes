use axum::Router;
use axum_login::{login_required, tower_sessions::MemoryStore, AuthManagerLayerBuilder};
use color_eyre::eyre::{Context, Result};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, SqlitePool};
use tower_sessions::{
    cookie::{time::Duration, Key},
    Expiry, SessionManagerLayer,
};

use crate::{config::Config, user::Backend};

use super::{auth, notes};

#[derive(Debug, Clone)]
pub struct App {
    pub config: Config,
    pub db: SqlitePool,
}

impl App {
    pub async fn new() -> Result<Self> {
        let config = Config::new()?;

        if !sqlx::Sqlite::database_exists(&config.db_url)
            .await
            .context("Could not get status of sqlite database.")?
        {
            sqlx::Sqlite::create_database(&config.db_url)
                .await
                .context("Could not create sqlite database.")?;
        }

        let db = SqlitePoolOptions::new()
            .connect(&config.db_url)
            .await
            .context("Could not connect to sqlite.")?;

        Ok(Self { config, db })
    }

    pub async fn serve(&self) -> Result<()> {
        sqlx::migrate!().run(&self.db).await?;

        let session_store = MemoryStore::default();

        // Generate a cryptographic key to sign the session cookie.
        let key = Key::generate();

        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_expiry(Expiry::OnInactivity(Duration::days(1)))
            .with_signed(key);

        let backend = Backend::new(self.db.clone());
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        let app = Router::new()
            .merge(notes::router().route_layer(login_required!(Backend, login_url = "/login")))
            .merge(auth::router())
            .layer(auth_layer)
            .with_state(self.clone());

        let listener = tokio::net::TcpListener::bind(format!(
            "{}:{}",
            self.config.bind_addr, self.config.port
        ))
        .await?;

        axum::serve(listener, app).await?;

        Ok(())
    }
}
