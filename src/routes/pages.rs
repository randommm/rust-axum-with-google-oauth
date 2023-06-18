use axum::{
    extract::{Extension, State},
    response::{Html, IntoResponse,},
    http::Request,
};

use minijinja::{context, Environment};

use super::UserData;

pub async fn index<T>(
    Extension(user_data): Extension<Option<UserData>>,
    State(env): State<Environment<'static>>,
    request: Request<T>,
) -> impl IntoResponse {
    let tmpl = env.get_template("index.html").unwrap();
    let user_email = user_data.map(|s| s.user_email);
    let login_return_url = "?return_url=".to_owned() + &*request.uri().to_string();
    let content = tmpl.render(context!(
        user_email => user_email,
        login_return_url => login_return_url,
    )).unwrap();
    Html(content)
}

pub async fn about<T>(
    Extension(user_data): Extension<Option<UserData>>,
    State(env): State<Environment<'static>>,
    request: Request<T>,
) -> impl IntoResponse {

    let tmpl = env.get_template("about.html").unwrap();
    let user_email = user_data.map(|s| s.user_email);
    let login_return_url = "?return_url=".to_owned() + &*request.uri().to_string();
    let content = tmpl.render(context!(
        user_email => user_email,
        login_return_url => login_return_url,
    )).unwrap();
    Html(content)
}

pub async fn profile(
    Extension(user_data): Extension<Option<UserData>>,
    State(env): State<Environment<'static>>,
) -> impl IntoResponse {
    let tmpl = env.get_template("profile.html").unwrap();
    let user_email = user_data.map(|s| s.user_email);
    let content = tmpl.render(context!(user_email => user_email)).unwrap();
    Html(content)
}
