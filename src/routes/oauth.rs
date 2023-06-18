// Code adapted from https://github.com/ramosbugs/oauth2-rs/blob/main/examples/google.rs
//
// Must set the enviroment variables:
// GOOGLE_CLIENT_ID=xxx
// GOOGLE_CLIENT_SECRET=yyy

use oauth2::{
    basic::BasicClient, TokenResponse,
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    RevocationUrl, Scope, TokenUrl, reqwest::http_client,
    PkceCodeVerifier
};
use dotenvy::var;
use axum::{
    extract::{Extension, State, Query, TypedHeader},
    headers::Cookie,
    http::StatusCode,
    response::{Redirect, IntoResponse},
};

use chrono::Utc;
use mongodb::{bson::{doc, Document}, Database};
use std::collections::HashMap;
use uuid::Uuid;

use super::UserData;

fn get_client() -> BasicClient {
    let google_client_id = ClientId::new(
        var("GOOGLE_CLIENT_ID").unwrap()
    );
    let google_client_secret = ClientSecret::new(
        var("GOOGLE_CLIENT_SECRET").unwrap()
    );
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .expect("Invalid token endpoint URL");

    // Set up the config for the Google OAuth2 process.
    BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:3011/oauth_return".to_string()).expect("Invalid redirect URL"),
    )
    .set_revocation_uri(
        RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
            .expect("Invalid revocation endpoint URL"),
    )
}

pub async fn login(
    Extension(user_data): Extension<Option<UserData>>,
    Query(mut params): Query<HashMap<String, String>>,
    State(database): State<Database>,
) -> Result<Redirect, StatusCode> {

    if user_data.is_some() { // check if already authenticated
        return Ok(Redirect::to("/"));
    }

    let return_url = params.remove("return_url").unwrap_or_else(|| "/".to_string());
    // TODO: check if return_url is valid

    let client = get_client();

    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.email".to_string(),
        ))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    // save csrf_state and pkce_code_verifier to database
    database.collection::<Document>("oauth2_state_storage")
        .insert_one(
            doc! {
                "csrf_state": csrf_state.secret(),
                "pkce_code_verifier": pkce_code_verifier.secret(),
                "return_url": return_url,
            },
            None,
        ).await.unwrap();

    Ok(Redirect::to(&*authorize_url.to_string()))
}

pub async fn oauth_return(
    Query(mut params): Query<HashMap<String, String>>,
    State(database): State<Database>,
) -> Result<impl IntoResponse, StatusCode> {

    let state = CsrfToken::new(params.remove("state").unwrap());
    let code = AuthorizationCode::new(params.remove("code").unwrap());

    // Given a csrf_state, get pkce_code_verifier and return_url from the database
    let oauth2_state_storage =
        database.collection::<Document>("oauth2_state_storage")
        .find_one_and_delete(doc! { "csrf_state": state.secret() }, None)
        .await
        .unwrap().unwrap();
    let pkce_code_verifier = oauth2_state_storage.get("pkce_code_verifier").unwrap().as_str().unwrap().to_owned();
    let return_url = oauth2_state_storage.get("return_url").unwrap().as_str().unwrap().to_owned();
    let pkce_code_verifier = PkceCodeVerifier::new(pkce_code_verifier);

    // Exchange the code with a token.
    let client = get_client();
    let token_response = tokio::task::spawn_blocking(move || {
        client
        .exchange_code(code)
        .set_pkce_verifier(pkce_code_verifier)
        .request(http_client).unwrap()
    }).await.unwrap();
    let access_token = token_response.access_token().secret();

    // Get user info from Google
    let url = "https://www.googleapis.com/oauth2/v2/userinfo?oauth_token=".to_owned() + access_token;
    let body = reqwest::get(url)
    .await.unwrap()
    .text()
    .await.unwrap();
    let mut body: serde_json::Value = serde_json::from_str(&*body).unwrap();
    let email = body["email"].take().as_str().unwrap().to_owned();
    let verified_email = body["verified_email"].take().as_bool().unwrap();
    assert_eq!(verified_email, true);

    // Check if user exists in database
    // If not, create a new user
    let user_id_query =
        database.collection::<Document>("users")
        .find_one(doc! { "email": &email }, None)
        .await
        .unwrap();
    let user_id =
    if let Some(user_id) = user_id_query {
        user_id.get_i32("_id").unwrap()
    } else {
        let user_id = database.collection::<Document>("counters")
        .find_one_and_update(
            doc! {"_id": "users"},
            doc! {"$inc": {"sequence_value":1}},
            mongodb::options::FindOneAndUpdateOptions::builder()
                .upsert(true)
                .return_document(mongodb::options::ReturnDocument::After)
                .build(),
        )
        .await
        .unwrap()
        .unwrap()
        .get_i32("sequence_value")
        .unwrap();

        database.collection::<Document>("users")
        .insert_one(
            doc! {
                "email": &email,
                "_id": &user_id,
            },
            None,
        ).await.unwrap();

        user_id
    };

    // Create a session for the user
    let session_token = Uuid::new_v4().to_string();
    let headers =
        axum::response::AppendHeaders([(
            axum::http::header::SET_COOKIE,
            "session_token=".to_owned() + &*session_token)
        ]);
    let now = Utc::now().timestamp();
    database.collection::<Document>("user_sessions")
    .insert_one(
        doc! {
            "session_token": session_token,
            "user_id": user_id,
            "created_at": now,
            "expires_at": now + 60*60*24,
        },
        None,
    ).await.unwrap();

    Ok((headers, Redirect::to(&*return_url)))
}

pub async fn logout(
    cookie: Option<TypedHeader<Cookie>>,
    State(database): State<Database>,
) -> Result<impl IntoResponse, StatusCode> {

    if let Some(cookie) = cookie {
        if let Some(session_token) = cookie.get("session_token") {
            database.collection::<Document>("user_sessions")
            .delete_many(
                doc! { "session_token": session_token},None,
            ).await.unwrap();
        }
    }
    let headers =
        axum::response::AppendHeaders([(
            axum::http::header::SET_COOKIE,
            "session_token=deleted; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT")
        ]);
    Ok((headers, Redirect::to("/")))
}
