use super::{AppError, UserData};
use axum::{
    extract::{State, TypedHeader},
    headers::Cookie,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Redirect},
};
use chrono::Utc;
use sqlx::SqlitePool;

pub async fn inject_user_data<T>(
    State(db_pool): State<SqlitePool>,
    cookie: Option<TypedHeader<Cookie>>,
    mut request: Request<T>,
    next: Next<T>,
) -> Result<impl IntoResponse, AppError> {
    if let Some(cookie) = cookie {
        if let Some(session_token) = cookie.get("session_token") {
            let query: Result<(i64, i64), _> = sqlx::query_as(
                r#"SELECT user_id,expires_at FROM user_sessions WHERE session_token=?"#,
            )
            .bind(session_token)
            .fetch_one(&db_pool)
            .await;

            if let Ok(query) = query {
                let user_id = query.0;
                let expires_at = query.1;
                if expires_at > Utc::now().timestamp() {
                    let query: Result<(String,), _> =
                        sqlx::query_as(r#"SELECT email FROM users WHERE id=?"#)
                            .bind(user_id)
                            .fetch_one(&db_pool)
                            .await;
                    if let Ok(query) = query {
                        let user_email = query.0;
                        request.extensions_mut().insert(Some(UserData {
                            user_id,
                            user_email,
                        }));
                    }
                }
            }
        }
    }

    Ok(next.run(request).await)
}

pub async fn check_auth<T>(
    request: Request<T>,
    next: Next<T>,
) -> Result<impl IntoResponse, AppError> {
    if request
        .extensions()
        .get::<Option<UserData>>()
        .ok_or("check_auth: extensions have no UserData")?
        .is_some()
    {
        Ok(next.run(request).await)
    } else {
        let login_url = "/login?return_url=".to_owned() + &*request.uri().to_string();
        Ok(Redirect::to(login_url.as_str()).into_response())
    }
}
