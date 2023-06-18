mod oauth;
mod middlewares;
mod pages;

use middlewares::{check_auth, inject_user_data};
use oauth::{login, oauth_return, logout};
use pages::{index, about, profile};

use minijinja::Environment;

use axum::{
    extract::FromRef,
    middleware,
    routing::get,
    Router,
    Extension,
};

use mongodb::Database;


#[derive(Clone, FromRef)]
pub struct AppState {
    pub database: Database,
    pub env: Environment<'static>,
}

#[derive(Clone, FromRef, Debug)]
pub struct UserData {
    pub user_id: i32,
    pub user_email: String,
}

pub async fn create_routes(database: Database) -> Router {
    let mut env = Environment::new();
    env.add_template("layout.html", include_str!("../templates/layout.html"))
        .unwrap();
    env.add_template("index.html", include_str!("../templates/index.html"))
        .unwrap();
    env.add_template("about.html", include_str!("../templates/about.html"))
        .unwrap();
    env.add_template("profile.html", include_str!("../templates/profile.html"))
        .unwrap();

    let app_state = AppState {
        database: database,
        env: env,
    };

    let user_data: Option<UserData> = None;

    Router::new()
        .route("/profile", get(profile))
        .route_layer(middleware::from_fn_with_state(app_state.clone(), check_auth))

        .route("/", get(index))
        .route("/about", get(about))
        .route("/login", get(login))
        .route("/oauth_return", get(oauth_return))
        .route("/logout", get(logout))

        .route_layer(middleware::from_fn_with_state(app_state.clone(), inject_user_data))

        .with_state(app_state)
        .layer(Extension(user_data))
}
