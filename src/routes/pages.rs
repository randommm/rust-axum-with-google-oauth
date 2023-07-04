use axum::{
    extract::{Extension, State},
    http::Request,
    response::{Html, IntoResponse},
};

use minijinja::{context, Environment};

use super::{AppError, UserData};

pub async fn index<T>(
    Extension(user_data): Extension<Option<UserData>>,
    State(env): State<Environment<'static>>,
    request: Request<T>,
) -> Result<impl IntoResponse, AppError> {
    let tmpl = env.get_template("index.html")?;
    let user_email = user_data.map(|s| s.user_email);
    let login_return_url = "?return_url=".to_owned() + &*request.uri().to_string();
    let content = tmpl.render(context!(
        user_email => user_email,
        login_return_url => login_return_url,
    ))?;
    Ok(Html(content))
}

pub async fn about<T>(
    Extension(user_data): Extension<Option<UserData>>,
    State(env): State<Environment<'static>>,
    request: Request<T>,
) -> Result<impl IntoResponse, AppError> {
    let tmpl = env.get_template("about.html")?;
    let user_email = user_data.map(|s| s.user_email);
    let login_return_url = "?return_url=".to_owned() + &*request.uri().to_string();
    let content = tmpl.render(context!(
        user_email => user_email,
        login_return_url => login_return_url,
    ))?;
    Ok(Html(content))
}

pub async fn profile(
    Extension(user_data): Extension<Option<UserData>>,
    State(env): State<Environment<'static>>,
) -> Result<impl IntoResponse, AppError> {
    let tmpl = env.get_template("profile.html")?;
    let user_email = user_data.map(|s| s.user_email);
    let content = tmpl.render(context!(user_email => user_email))?;
    Ok(Html(content))
}
