use askama_axum::Template;
use axum::{extract::{Query, State}, response::IntoResponse, routing::{get, post}, Json, Router};
use color_eyre::eyre::{Context, Result};
use eos::{ext::IntervalLiteral, fmt::format_spec, DateTime, Timestamp, Utc};
use serde::Deserialize;
use sqlx::{prelude::FromRow, sqlite::SqlitePoolOptions, SqlitePool};
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
struct Note {
    created_at: i64,
    content: String,
    id: String,
}

impl Note {
    fn format(&self) -> String {
        self.get_datetime()
            .format(format_spec!("%I:%M %p"))
            .to_string()
    }

    fn get_datetime(&self) -> DateTime {
        eos::DateTime::from_timestamp(Timestamp::from_milliseconds(self.created_at), Utc)
    }
}

#[derive(Debug, Deserialize, Clone)]
struct Config {
    port: u32,
    bind_addr: String,
    db_url: String,
}

#[derive(Debug, Clone)]
struct AppState {
    config: Config,
    db: SqlitePool,
}

#[derive(Debug, Deserialize)]
struct CreateArgs {
    #[serde(rename = "note-input")]
    text: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate();

#[derive(Template)]
#[template(path = "notes.html")]
struct NotesTemplates {
    groups: Vec<Vec<Note>>,
    page: u32,
    per_page: u32
}

#[derive(Deserialize)]
struct Pagination {
    page: u32,
    per_page: u32
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let config: Config = ron::from_str(
        &std::fs::read_to_string("./config.ron").context("Failed to open configuration file.")?,
    )
    .context("Failed to parse configuration.")?;
    let db = SqlitePoolOptions::new()
        .connect(&config.db_url)
        .await
        .context("Could not connect to sqlite.")?;

    sqlx::migrate!().run(&db).await?;

    let state = AppState {
        config: config.clone(),
        db,
    };

    let app: Router = Router::new()
        .route("/", get(index))
        .route("/notes", get(get_notes))
        .route("/create-note", post(create_note))
        .with_state(state);

    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.bind_addr, config.port)).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn index() -> impl IntoResponse {
    IndexTemplate {}
}

async fn get_notes(State(state): State<AppState>, pagination: Query<Pagination>) -> impl IntoResponse {
    let pagination: Pagination = pagination.0;
    let mut notes: Vec<Note> =
        sqlx::query_as::<_, Note>("SELECT * FROM notes ORDER BY created_at DESC LIMIT ? OFFSET ?")
            .bind(pagination.per_page)
            .bind(pagination.per_page * pagination.page)
            .fetch_all(&state.db)
            .await
            .unwrap();
    notes.reverse();

    if notes.len() == 0 {
        return NotesTemplates {
            groups: vec![],
            page: pagination.page,
            per_page: pagination.per_page
        }
    }

    let mut groups: Vec<Vec<Note>> = Vec::new();
    let mut current: Vec<Note> = vec![notes[0].clone()];
    let mut current_max = current[0].get_datetime() + 15.minutes();

    for i in 1..(notes.len()) {
        if notes[i].get_datetime() < current_max {
            current.push(notes[i].clone());
        } else {
            groups.push(current);
            current = vec![notes[i].clone()];
            current_max = current[0].get_datetime() + 15.minutes();
        }
    }
    groups.push(current);
    groups.reverse();

    NotesTemplates { groups, page: pagination.page, per_page: pagination.per_page }
}

async fn create_note(
    State(state): State<AppState>,
    Json(data): Json<CreateArgs>,
) -> impl IntoResponse {
    let id = Uuid::new_v4();
    let created_at = DateTime::utc_now().timestamp().as_milliseconds();

    sqlx::query("INSERT INTO notes (created_at, content, id) VALUES(?, ?, ?)")
        .bind(created_at as i64)
        .bind(data.text)
        .bind(id.to_string())
        .execute(&state.db)
        .await
        .unwrap();
}
