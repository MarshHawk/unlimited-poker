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

pub mod schema {
    include!("./schema/mod.rs");
}
use schema::{
    mutation::MutationRoot, HandToken, PokerSchema, QueryRoot, Storage, SubscriptionRoot,
    TableToken, UserToken,
};

use schema::deal::dealer_client::DealerClient;

fn get_user_token_from_headers(headers: &HeaderMap) -> Option<UserToken> {
    headers
        .get("x-user-token")
        .and_then(|value| value.to_str().map(|s| UserToken(s.to_string())).ok())
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
    // println!("{:?}", value);

    let mut data = Data::default();

    if let Some(user_token) = value
        .get("x-user-token")
        .and_then(|user_token| user_token.as_str())
    {
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

pub async fn bootstrap_schema() -> Result<Schema<QueryRoot, MutationRoot, SubscriptionRoot>, Box<dyn std::error::Error>> {
    let deal_client: schema::DealService = Arc::new(Mutex::new(
        DealerClient::connect("http://127.0.0.1:5003").await?,
    ));
    Ok(Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(Storage::default())
        .data(deal_client)
        .finish())
}

pub fn bootstrap(cfg: &mut web::ServiceConfig) {
    println!("Bootstrap GraphiQL IDE: http://localhost:8097");

    cfg
        .service(web::resource("/graphql").guard(guard::Post()).to(index))
        .service(
            web::resource("/ws")
                .guard(guard::Get())
                .guard(guard::Header("upgrade", "websocket"))
                .to(index_ws),
        )
        .service(graphql_playground);
}
