use rust_axum_with_google_oauth::run;
use dotenvy::var;

#[tokio::main]
async fn main() {
    let database_uri = var("DATABASE_URI").unwrap();
    run(database_uri).await;
}
