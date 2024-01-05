use std::sync::Arc;

use actix_web::{
    get, guard, http::header::HeaderMap, web, App, HttpRequest, HttpResponse, HttpServer,
    Responder, Result,
};
use actix_web_lab::respond::Html;
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Data, Schema,
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use futures_util::lock::Mutex;

//mod schema;
//use schema::{
//    mutation::MutationRoot, QueryRoot, Storage, SubscriptionRoot
//};
//
//use schema::deal::dealer_client::DealerClient;


mod bootstrap;
use bootstrap::{bootstrap, bootstrap_schema};

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
