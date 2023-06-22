use rust_axum_with_google_oauth::run;
use dotenvy::var;

#[tokio::main]
async fn main() -> Result<(), String> {
    let database_uri = var("DATABASE_URI").map_err(|e| {
        format!("Failed to get DATABASE_URI: {}", e)
    })?;
    run(database_uri).await
}
