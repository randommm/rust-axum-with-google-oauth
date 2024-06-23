mod error_handling;
mod middlewares;
mod oauth;
mod pages;

use error_handling::AppError;
use middlewares::{check_auth, inject_user_data};
use oauth::{login, logout, oauth_return};
use pages::{about, index, profile};

use axum::{extract::FromRef, middleware, routing::get, Extension, Router};
use minijinja::Environment;
use sqlx::SqlitePool;
use std::fs;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db_pool: SqlitePool,
    pub env: Environment<'static>,
}

#[derive(Clone, Debug)]
pub struct UserData {
    #[allow(dead_code)]
    pub user_id: i64,
    pub user_email: String,
}

pub async fn create_routes(db_pool: SqlitePool) -> Result<Router, Box<dyn std::error::Error>> {
    let mut env = Environment::new();

    let paths = fs::read_dir("src/templates").unwrap();
    for path in paths {
        let path = path.map_err(|e| format!("Error on file {e}"))?.path();
        let source = fs::read_to_string(&path)?;
        let path = path.to_str().ok_or("Failed to convert path to str")?;
        let path = &path[14..];
        env.add_template_owned(path.to_owned(), source)
            .map_err(|e| format!("Failed to add {path}: {e}"))?;
    }

    let app_state = AppState { db_pool, env };

    let user_data: Option<UserData> = None;

    Ok(Router::new()
        .route("/profile", get(profile))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            check_auth,
        ))
        .route("/", get(index))
        .route("/about", get(about))
        .route("/login", get(login))
        .route("/oauth_return", get(oauth_return))
        .route("/logout", get(logout))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            inject_user_data,
        ))
        .with_state(app_state)
        .layer(Extension(user_data)))
}
