use color_eyre::Result;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::web::App;

mod config;
mod filters;
mod models;
mod user;
mod web;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let app = App::new().await?;

    app.serve().await?;

    Ok(())
}

// async fn index() -> impl IntoResponse {
//     IndexTemplate {}
// }

// async fn create_note(
//     State(state): State<AppState>,
//     Json(data): Json<CreateArgs>,
// ) -> impl IntoResponse {
//     let id = Uuid::new_v4();
//     let created_at = DateTime::utc_now().timestamp().as_milliseconds();

//     sqlx::query("INSERT INTO notes (created_at, content, id) VALUES(?, ?, ?)")
//         .bind(created_at as i64)
//         .bind(data.text)
//         .bind(id.to_string())
//         .execute(&state.db)
//         .await
//         .unwrap();
// }
