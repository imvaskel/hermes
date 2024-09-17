use askama::Template;
use axum::{
    routing::{get, post},
    Router,
};
use serde::Deserialize;

use super::App;

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    message: Option<String>,
}

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {
    message: Option<String>,
}

#[derive(Deserialize)]
pub struct RegisterForm {
    username: String,
    password: String,
    access_key: String,
}

pub fn router() -> Router<App> {
    Router::new()
        .route("/login", get(self::get::login).post(self::post::login))
        .route("/logout", get(self::get::logout))
        .route(
            "/register",
            get(self::get::register).post(self::post::register),
        )
}

mod post {
    use axum::{
        extract::State,
        http::StatusCode,
        response::{IntoResponse, Redirect},
        Form,
    };
    use uuid::Uuid;

    use crate::{
        user::{AuthSession, Credentials},
        web::App,
    };

    use super::*;

    pub async fn login(
        mut auth_session: AuthSession,
        Form(creds): Form<Credentials>,
    ) -> impl IntoResponse {
        let user = match auth_session.authenticate(creds.clone()).await {
            Ok(Some(u)) => u,
            Ok(None) => {
                return LoginTemplate {
                    message: Some("Invalid username/password.".into()),
                }
                .into_response()
            }
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        if let Some(ref next) = creds.next {
            Redirect::to(next).into_response()
        } else {
            Redirect::to("/").into_response()
        }
    }

    pub async fn register(
        mut auth_session: AuthSession,
        State(state): State<App>,
        Form(form): Form<RegisterForm>,
    ) -> impl IntoResponse {
        if state.config.registration_key != form.access_key {
            return RegisterTemplate {
                message: Some("Invalid access key".into()),
            }
            .into_response();
        }

        let pw = password_auth::generate_hash(&form.password);
        let id = Uuid::new_v4();
        match sqlx::query("INSERT INTO users VALUES($1, $2, $3)")
            .bind(&id.to_string())
            .bind(&form.username)
            .bind(&pw)
            .execute(&state.db)
            .await
        {
            Ok(_) => {
                let creds = Credentials {
                    username: form.username,
                    password: pw,
                    next: None,
                };
                self::login(auth_session, axum::Form(creds))
                    .await
                    .into_response()
            }
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

mod get {
    use axum::{
        http::StatusCode,
        response::{IntoResponse, Redirect},
    };

    use crate::user::AuthSession;

    use super::*;

    pub async fn login() -> impl IntoResponse {
        LoginTemplate { message: None }.into_response()
    }

    pub async fn register() -> impl IntoResponse {
        RegisterTemplate { message: None }.into_response()
    }

    pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
        match auth_session.logout().await {
            Ok(_) => Redirect::to("/login").into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
