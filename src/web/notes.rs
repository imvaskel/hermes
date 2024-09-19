use askama::Template;
use axum::{
    routing::{delete, get, post},
    Router,
};
use serde::Deserialize;

use crate::models::{Entry, Note};

use super::App;

#[derive(Template)]
#[template(path = "notes.html")]
struct EntriesTemplate {
    pub notes: Vec<Entry>,
    pub page: u32,
    pub per_page: u32,
    pub note_id: String,
}

#[derive(Debug, Deserialize)]
struct CreateEntryForm {
    #[serde(rename = "note-input")]
    pub text: String,
}

#[derive(Debug, Deserialize)]
struct CreateNoteForm {
    pub name: String,
}
#[derive(Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
}

#[derive(Template)]
#[template(path = "note.html")]
struct NoteTemplate {
    id: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    notes: Vec<Note>,
}

pub fn router() -> Router<App> {
    Router::new()
        .route("/", get(self::get::index))
        .route("/notes/create", post(self::post::create_note))
        .route("/notes/:id", get(self::get::note))
        .route("/notes/:id", delete(self::delete::note))
        .route("/notes/:id/entries", get(self::get::entries))
        .route("/notes/:id/entries/create", post(self::post::create))
}

mod post {
    use askama_axum::IntoResponse;
    use axum::{
        extract::{Path, State},
        http::StatusCode,
        response::Redirect,
        Form,
    };
    use eos::DateTime;

    use tracing::debug;
    use uuid::Uuid;

    use super::*;
    use crate::{models::Note, user::AuthSession, web::App};

    pub async fn create(
        State(state): State<App>,
        auth: AuthSession,
        Path(id): Path<String>,
        Form(form): Form<CreateEntryForm>,
    ) -> impl IntoResponse {
        let user = match auth.user {
            Some(u) => u,
            None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        let note: Note = match sqlx::query_as(
            r#"
            SELECT * FROM notes
            JOIN users ON notes.note_owner = users.user_id
            WHERE notes.note_id = $1
            AND users.user_id = $2
        "#,
        )
        .bind(id.clone())
        .bind(user.id.clone())
        .fetch_optional(&state.db)
        .await
        {
            Ok(Some(n)) => n,
            Ok(None) => return StatusCode::NOT_FOUND.into_response(),
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        let id = Uuid::new_v4().to_string();
        let now = DateTime::utc_now().timestamp().as_milliseconds() as i64;

        match sqlx::query(
            r#"INSERT INTO entries(entry_id, created_at, content, parent) VALUES($1, $2, $3, $4)"#,
        )
        .bind(id.clone())
        .bind(now)
        .bind(form.text)
        .bind(note.id)
        .execute(&state.db)
        .await
        {
            Ok(_) => {
                let entry: Entry = match sqlx::query_as(
                    r#"SELECT * FROM entries
                JOIN notes ON entries.parent = notes.note_id
                JOIN users ON notes.note_owner = users.user_id
                WHERE entries.entry_id = $1"#,
                )
                .bind(id)
                .fetch_one(&state.db)
                .await
                {
                    Ok(n) => n,
                    Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                };
                EntriesTemplate {
                    note_id: entry.id.clone(),
                    notes: vec![entry],
                    page: 0,
                    per_page: 5,
                }
                .into_response()
            }
            .into_response(),
            Err(e) => {
                debug!("{:#?}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }

    pub async fn create_note(
        State(state): State<App>,
        auth: AuthSession,
        Form(form): Form<CreateNoteForm>,
    ) -> impl IntoResponse {
        let user = match auth.user {
            Some(u) => u,
            None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        let id = Uuid::new_v4().to_string();

        match sqlx::query(
            r#"INSERT INTO notes(note_id, note_name, note_owner)
        VALUES($1, $2, $3) "#,
        )
        .bind(id.clone())
        .bind(form.name)
        .bind(user.id)
        .execute(&state.db)
        .await
        {
            Ok(_) => Redirect::to(&format!("/notes/{}", id)).into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

mod delete {

    use askama_axum::IntoResponse;
    use axum::{
        body::Body,
        extract::{Path, State},
        http::{Response, StatusCode},
    };
    use axum_htmx::HX_REFRESH;

    use crate::user::AuthSession;

    use super::*;

    pub async fn note(
        State(state): State<App>,
        auth: AuthSession,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let user = match auth.user {
            Some(u) => u,
            None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        match sqlx::query("DELETE FROM notes WHERE note_id = $1 AND note_owner = $2")
            .bind(id)
            .bind(user.id)
            .execute(&state.db)
            .await
        {
            Ok(_) => Response::builder()
                .header(HX_REFRESH, "true")
                .status(StatusCode::OK)
                .body(Body::default())
                .unwrap(),
            Err(_) => StatusCode::FORBIDDEN.into_response(),
        }
    }
}

mod get {
    use askama_axum::IntoResponse;
    use axum::{
        extract::{Path, Query, State},
        http::StatusCode,
    };
    
    
    

    use crate::{models::Note, user::AuthSession, web::App};

    use super::*;

    pub async fn index(State(state): State<App>, auth_state: AuthSession) -> impl IntoResponse {
        let user = match auth_state.user {
            Some(u) => u,
            None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        let user_notes: Vec<Note> = (sqlx::query_as(
            r#"
        SELECT * FROM notes
        JOIN users ON notes.note_owner = users.user_id
        WHERE notes.note_owner = $1 AND users.user_id = $1"#,
        )
        .bind(user.id)
        .fetch_all(&state.db)
        .await).unwrap_or_default();

        IndexTemplate { notes: user_notes }.into_response()
    }

    pub async fn note(Path(id): Path<String>) -> impl IntoResponse {
        NoteTemplate { id }.into_response()
    }

    pub async fn entries(
        auth_state: AuthSession,
        State(state): State<App>,
        Path(id): Path<String>,
        pagination: Query<Pagination>,
    ) -> impl IntoResponse {
        let user = match auth_state.user {
            Some(u) => u,
            None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        let pagination: Pagination = pagination.0;
        let mut notes: Vec<Entry> = sqlx::query_as::<_, Entry>(
            r#"SELECT * FROM entries
            JOIN notes ON entries.parent = notes.note_id
            JOIN users ON notes.note_owner = users.user_id
            WHERE parent = $1 AND notes.note_owner = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4"#,
        )
        .bind(&id)
        .bind(&user.id)
        .bind(pagination.per_page)
        .bind(pagination.per_page * pagination.page)
        .fetch_all(&state.db)
        .await
        .unwrap();
        notes.reverse();

        if notes.is_empty() {
            return EntriesTemplate {
                notes: vec![],
                page: pagination.page,
                per_page: pagination.per_page,
                note_id: "".into(),
            }
            .into_response();
        }

        EntriesTemplate {
            note_id: notes[0].parent.id.clone(),
            notes,
            page: pagination.page,
            per_page: pagination.per_page,
        }
        .into_response()
    }
}
