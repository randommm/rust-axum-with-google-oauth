mod error_handling;
mod middlewares;
mod oauth;
mod pages;

use error_handling::AppError;
use middlewares::{check_auth, inject_user_data};
use oauth::{login, logout, oauth_return};
use pages::{about, index, profile};

use minijinja::Environment;

use axum::{extract::FromRef, middleware, routing::get, Extension, Router};

use mongodb::Database;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub database: Database,
    pub env: Environment<'static>,
}

#[derive(Clone, Debug)]
pub struct UserData {
    pub user_id: i32,
    pub user_email: String,
}

pub async fn create_routes(database: Database) -> Result<Router, String> {
    let mut env = Environment::new();
    env.add_template("layout.html", include_str!("../templates/layout.html"))
        .map_err(|e| format!("Failed to add layout.html: {}", e))?;

    env.add_template("index.html", include_str!("../templates/index.html"))
        .map_err(|e| format!("Failed to add index.html: {}", e))?;

    env.add_template("about.html", include_str!("../templates/about.html"))
        .map_err(|e| format!("Failed to add about.html: {}", e))?;

    env.add_template("profile.html", include_str!("../templates/profile.html"))
        .map_err(|e| format!("Failed to add profile.html: {}", e))?;

    let app_state = AppState { database, env };

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
