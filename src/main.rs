use actix_web::{web, App, HttpServer,Result,};

use unlimited_poker::bootstrap::{bootstrap, bootstrap_schema};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Launching App from main");

    let schema = bootstrap_schema().await.unwrap();

    let _ = HttpServer::new(move || App::new().app_data(web::Data::new(schema.clone())).configure(bootstrap))
        .bind("0.0.0.0:8097")?
        .run()
        .await?;

    Ok(())
}