use dotenvy::var;
use rust_axum_with_google_oauth_mongodb::run;

#[tokio::main]
async fn main() -> Result<(), String> {
    let database_uri =
        var("DATABASE_URL").map_err(|e| format!("Failed to get DATABASE_URI: {}", e))?;
    run(database_uri).await
}
