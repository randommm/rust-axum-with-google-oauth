// Code adapted from https://github.com/ramosbugs/oauth2-rs/blob/main/examples/google.rs
//
// Must set the enviroment variables:
// GOOGLE_CLIENT_ID=xxx
// GOOGLE_CLIENT_SECRET=yyy

use axum::{
    extract::{Extension, Host, Query, State, TypedHeader},
    headers::Cookie,
    response::{IntoResponse, Redirect},
};
use dotenvy::var;
use oauth2::{
    basic::BasicClient, reqwest::http_client, AuthUrl, AuthorizationCode, ClientId, ClientSecret,
    CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RevocationUrl, Scope,
    TokenResponse, TokenUrl,
};

use chrono::Utc;
use mongodb::{
    bson::{doc, Document},
    Database,
};
use std::collections::HashMap;
use uuid::Uuid;

use super::{AppError, UserData};

fn get_client(hostname: String) -> Result<BasicClient, AppError> {
    let google_client_id = ClientId::new(var("GOOGLE_CLIENT_ID")?);
    let google_client_secret = ClientSecret::new(var("GOOGLE_CLIENT_SECRET")?);
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .map_err(|_| "OAuth: invalid authorization endpoint URL")?;
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .map_err(|_| "OAuth: invalid token endpoint URL")?;

    let protocol = if hostname.starts_with("localhost") || hostname.starts_with("127.0.0.1") {
        "http"
    } else {
        "https"
    };

    let redirect_url = format!("{}://{}/oauth_return", protocol, hostname);

    // Set up the config for the Google OAuth2 process.
    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).map_err(|_| "OAuth: invalid redirect URL")?)
    .set_revocation_uri(
        RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
            .map_err(|_| "OAuth: invalid revocation endpoint URL")?,
    );
    Ok(client)
}

pub async fn login(
    Extension(user_data): Extension<Option<UserData>>,
    Query(mut params): Query<HashMap<String, String>>,
    State(database): State<Database>,
    Host(hostname): Host,
) -> Result<Redirect, AppError> {
    if user_data.is_some() {
        // check if already authenticated
        return Ok(Redirect::to("/"));
    }

    let return_url = params
        .remove("return_url")
        .unwrap_or_else(|| "/".to_string());
    // TODO: check if return_url is valid

    let client = get_client(hostname)?;

    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.email".to_string(),
        ))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    // save csrf_state and pkce_code_verifier to database
    database
        .collection::<Document>("oauth2_state_storage")
        .insert_one(
            doc! {
                "csrf_state": csrf_state.secret(),
                "pkce_code_verifier": pkce_code_verifier.secret(),
                "return_url": return_url,
            },
            None,
        )
        .await?;

    Ok(Redirect::to(authorize_url.as_str()))
}

pub async fn oauth_return(
    Query(mut params): Query<HashMap<String, String>>,
    State(database): State<Database>,
    Host(hostname): Host,
) -> Result<impl IntoResponse, AppError> {
    let state = CsrfToken::new(params.remove("state").ok_or("OAuth: without state")?);
    let code = AuthorizationCode::new(params.remove("code").ok_or("OAuth: without code")?);

    // Given a csrf_state, get pkce_code_verifier and return_url from the database
    let oauth2_state_storage = database
        .collection::<Document>("oauth2_state_storage")
        .find_one_and_delete(doc! { "csrf_state": state.secret() }, None)
        .await?
        .ok_or("OAuth: csrf_state not found on DB")?;
    let pkce_code_verifier = oauth2_state_storage
        .get("pkce_code_verifier")
        .ok_or("OAuth: pkce_code_verifier not found on DB")?
        .as_str()
        .ok_or("OAuth: pkce_code_verifier is not a str on DB")?
        .to_owned();
    let return_url = oauth2_state_storage
        .get("return_url")
        .ok_or("OAuth: return_url not found on DB")?
        .as_str()
        .ok_or("OAuth: return_url is not a str on DB")?
        .to_owned();
    let pkce_code_verifier = PkceCodeVerifier::new(pkce_code_verifier);

    // Exchange the code with a token.
    let client = get_client(hostname)?;
    let token_response = tokio::task::spawn_blocking(move || {
        client
            .exchange_code(code)
            .set_pkce_verifier(pkce_code_verifier)
            .request(http_client)
    })
    .await
    .map_err(|_| "OAuth: exchange_code failure")?
    .map_err(|_| "OAuth: tokio spawn blocking failure")?;
    let access_token = token_response.access_token().secret();

    // Get user info from Google
    let url =
        "https://www.googleapis.com/oauth2/v2/userinfo?oauth_token=".to_owned() + access_token;
    let body = reqwest::get(url)
        .await
        .map_err(|_| "OAuth: reqwest failed to query userinfo")?
        .text()
        .await
        .map_err(|_| "OAuth: reqwest received invalid userinfo")?;
    let mut body: serde_json::Value =
        serde_json::from_str(body.as_str()).map_err(|_| "OAuth: Serde failed to parse userinfo")?;
    let email = body["email"]
        .take()
        .as_str()
        .ok_or("OAuth: Serde failed to parse email address")?
        .to_owned();
    let verified_email = body["verified_email"]
        .take()
        .as_bool()
        .ok_or("OAuth: Serde failed to parse verified_email")?;
    if !verified_email {
        return Err(AppError::new("OAuth: email address is not verified".to_owned())
            .with_user_message("Your email address is not verified. Please verify your email address with Google and try again.".to_owned()));
    }

    // Check if user exists in database
    // If not, create a new user
    let user_id_query = database
        .collection::<Document>("users")
        .find_one(doc! { "email": &email }, None)
        .await?;
    let user_id = if let Some(user_id) = user_id_query {
        user_id.get_i32("_id")?
    } else {
        let user_id = database
            .collection::<Document>("counters")
            .find_one_and_update(
                doc! {"_id": "users"},
                doc! {"$inc": {"sequence_value":1}},
                mongodb::options::FindOneAndUpdateOptions::builder()
                    .upsert(true)
                    .return_document(mongodb::options::ReturnDocument::After)
                    .build(),
            )
            .await?
            .ok_or("OAuth: failed to execute find_one_and_update for sequence_value for users")?
            .get_i32("sequence_value")?;

        database
            .collection::<Document>("users")
            .insert_one(
                doc! {
                    "email": &email,
                    "_id": &user_id,
                },
                None,
            )
            .await?;

        user_id
    };

    // Create a session for the user
    let session_token = Uuid::new_v4().to_string();
    let headers = axum::response::AppendHeaders([(
        axum::http::header::SET_COOKIE,
        "session_token=".to_owned() + &*session_token,
    )]);
    let now = Utc::now().timestamp();
    database
        .collection::<Document>("user_sessions")
        .insert_one(
            doc! {
                "session_token": session_token,
                "user_id": user_id,
                "created_at": now,
                "expires_at": now + 60*60*24,
            },
            None,
        )
        .await?;

    Ok((headers, Redirect::to(return_url.as_str())))
}

pub async fn logout(
    cookie: Option<TypedHeader<Cookie>>,
    State(database): State<Database>,
) -> Result<impl IntoResponse, AppError> {
    if let Some(cookie) = cookie {
        if let Some(session_token) = cookie.get("session_token") {
            database
                .collection::<Document>("user_sessions")
                .delete_many(doc! { "session_token": session_token}, None)
                .await?;
        }
    }
    let headers = axum::response::AppendHeaders([(
        axum::http::header::SET_COOKIE,
        "session_token=deleted; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT",
    )]);
    Ok((headers, Redirect::to("/")))
}
