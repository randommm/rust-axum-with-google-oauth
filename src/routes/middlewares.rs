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
            let session_token: Vec<&str> = session_token.split('_').collect();
            let query: Result<(i64, i64, String), _> = sqlx::query_as(
                r#"SELECT user_id,expires_at,session_token_p2 FROM user_sessions WHERE session_token_p1=?"#,
            )
            .bind(session_token[0])
            .fetch_one(&db_pool)
            .await;

            if let Ok(query) = query {
                if let Ok(session_token_p2_db) = query.2.as_bytes().try_into() {
                    if let Ok(session_token_p2_cookie) = session_token
                        .get(1)
                        .copied()
                        .unwrap_or_default()
                        .as_bytes()
                        .try_into()
                    {
                        if constant_time_eq::constant_time_eq_n::<36>(
                            session_token_p2_cookie,
                            session_token_p2_db,
                        ) {
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
