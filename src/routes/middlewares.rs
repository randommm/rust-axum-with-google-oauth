use super::{AppError, UserData};
use axum::{
    extract::{State, TypedHeader},
    headers::Cookie,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Redirect},
};
use chrono::Utc;
use mongodb::{
    bson::{doc, Document},
    Database,
};

pub async fn inject_user_data<T>(
    State(database): State<Database>,
    cookie: Option<TypedHeader<Cookie>>,
    mut request: Request<T>,
    next: Next<T>,
) -> Result<impl IntoResponse, AppError> {
    if let Some(cookie) = cookie {
        if let Some(session_token) = cookie.get("session_token") {
            let session_token: Vec<&str> = session_token.split('_').collect();
            let user_session = database
                .collection::<Document>("user_sessions")
                .find_one(doc! {"session_token_p1": session_token[0]}, None)
                .await?;
            if let Some(user_session) = user_session {
                if let Ok(session_token_2_db) = user_session.get_str("session_token_p2") {
                    if let Ok(session_token_p2_db) = session_token_2_db.as_bytes().try_into() {
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
                                // document exists
                                if let Ok(expires_at) = user_session.get_i64("expires_at") {
                                    // document has expires_at
                                    if expires_at > Utc::now().timestamp() {
                                        // session not expired
                                        if let Ok(user_id) = user_session.get_i32("user_id") {
                                            // document has user_id
                                            let user_email = database
                                                .collection::<Document>("users")
                                                .find_one(doc! {"_id": user_id}, None)
                                                .await?
                                                .ok_or("inject_user_data: user not found on DB")?;
                                            let user_email =
                                                user_email.get_str("email")?.to_owned();
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
