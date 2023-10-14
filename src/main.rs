use dotenvy::var;
use rust_axum_with_google_oauth::run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url =
        var("DATABASE_URL").map_err(|e| format!("Failed to get DATABASE_URL: {}", e))?;
    run(database_url).await
}
