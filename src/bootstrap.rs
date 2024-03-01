use actix_web::{
    get, guard, http::header::HeaderMap, web, HttpRequest, HttpResponse, Responder, Result,
};
use actix_web_lab::respond::Html;
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Data, Schema,
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use darkbird::{
    document::{self, RangeField},
    Options, Storage, StorageType,
};
//use serde_derive::{Deserialize, Serialize};
use mongodb::{options::ClientOptions, Client};
use mongodb::options::Credential;
use serde::{Deserialize, Serialize};
use std::env;

use crate::bootstrap::schema::model::Hand;
pub mod schema {
    include!("./schema/mod.rs");
}
use schema::{
    mutation::MutationRoot, HandToken, PokerSchema, QueryRoot, SubscriptionRoot, TableToken,
    UserToken,
};

fn get_user_token_from_headers(headers: &HeaderMap) -> Option<UserToken> {
    let token = headers
        .get("x-user-token")
        .and_then(|value| value.to_str().map(|s| UserToken(s.to_string())).ok());
    println!("get user token from headers");
    println!("token: {:?}", token);
    token
}

fn get_table_token_from_headers(headers: &HeaderMap) -> Option<TableToken> {
    headers
        .get("x-table-token")
        .and_then(|value| value.to_str().map(|s| TableToken(s.to_string())).ok())
}

fn get_hand_token_from_headers(headers: &HeaderMap) -> Option<HandToken> {
    headers
        .get("x-hand-token")
        .and_then(|value| value.to_str().map(|s| HandToken(s.to_string())).ok())
}

pub async fn on_connection_init(value: serde_json::Value) -> async_graphql::Result<Data> {
    println!("on_connection_init");
    println!("{:?}", value);

    let mut data = Data::default();

    if let Some(user_token) = value
        .get("x-user-token")
        .and_then(|user_token| user_token.as_str())
    {
        println!("insert user token");
        data.insert(UserToken(user_token.to_string()));
    }

    if let Some(table_token) = value
        .get("x-table-token")
        .and_then(|table_token| table_token.as_str())
    {
        data.insert(TableToken(table_token.to_string()));
    }

    if let Some(hand_token) = value
        .get("x-hand-token")
        .and_then(|hand_token| hand_token.as_str())
    {
        data.insert(HandToken(hand_token.to_string()));
    }

    //TODO: handle missing tokens

    Ok(data)
}

async fn index_ws(
    schema: web::Data<PokerSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema))
        .on_connection_init(on_connection_init)
        .start(&req, payload)
}

async fn index(
    schema: web::Data<PokerSchema>,
    req: HttpRequest,
    gql_request: GraphQLRequest,
) -> GraphQLResponse {
    // println!("req: {:?}", req);
    let mut request = gql_request.into_inner();
    if let Some(token) = get_table_token_from_headers(req.headers()) {
        request = request.data(token);
    }
    if let Some(token) = get_user_token_from_headers(req.headers()) {
        request = request.data(token);
    }
    if let Some(token) = get_hand_token_from_headers(req.headers()) {
        request = request.data(token);
    }
    schema.execute(request).await.into()
}

/// GraphQL endpoint
/* #[route("/graphql", method = "GET", method = "POST")]
async fn graphql(schema: web::Data<PokerSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
} */

/// GraphiQL playground UI
#[get("/graphiql")]
async fn graphql_playground() -> impl Responder {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/ws"),
    ))
}

pub async fn bootstrap_schema(
) -> Result<Schema<QueryRoot, MutationRoot, SubscriptionRoot>, Box<dyn std::error::Error>> {
    let path = ".";
    let storage_name = "blackbird";
    let total_page_size = 1000;
    let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    let creds = Credential::builder()
    .username("root".to_string())
    .password("example".to_string())
    .build();
    client_options.credential = Some(creds.clone());
    println!("Username: {}, Password: {}", creds.username.unwrap(), creds.password.unwrap());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options)?;
    let db = client.database("poker");
    //let hands_collection = db.collection("hands");
    let stype = StorageType::RamCopies;
    let ops = Options::new(
        path,
        storage_name,
        total_page_size,
        StorageType::RamCopies,
        true,
    );
    let storage = Storage::<String, Hand>::open(ops).await.unwrap();
    Ok(Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(storage)
        .data(db)
        // .data(deal_client)
        .finish())
}

pub fn bootstrap(cfg: &mut web::ServiceConfig) {
    println!("Bootstrap GraphiQL IDE: http://localhost:8097");

    cfg.service(web::resource("/graphql").guard(guard::Post()).to(index))
        .service(
            web::resource("/ws")
                .guard(guard::Get())
                .guard(guard::Header("upgrade", "websocket"))
                .to(index_ws),
        )
        .service(graphql_playground);
}
