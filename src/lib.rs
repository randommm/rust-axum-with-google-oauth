mod routes;

use mongodb::{
    bson::doc,
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client,
};

async fn mongo_connect(database_uri: String) -> mongodb::error::Result<Client> {
    let mut client_options = ClientOptions::parse(database_uri).await?;
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    let client = Client::with_options(client_options)?;
    client
        .database("admin")
        .run_command(doc! {"ping": 1}, None)
        .await?;
    println!("Successfully connected to MongoDB!");
    Ok(client)
}

pub async fn run(database_uri: String) -> Result<(), String> {
    let client = mongo_connect(database_uri)
        .await
        .map_err(|e| format!("Failed to connect to MongoDB: {}", e))?;
    let database = client.database("axumApp");
    let app = routes::create_routes(database).await?;
    let bind_addr = &"0.0.0.0:3011"
        .parse()
        .map_err(|e| format!("Failed to parse address: {}", e))?;
    axum::Server::bind(bind_addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| format!("Server error: {}", e))
}
