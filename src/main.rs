use std::sync::{Arc};

use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer, Result, Responder, get, route};
use async_graphql::{http::{GraphQLPlaygroundConfig, playground_source}, Schema};
use async_graphql_actix_web::{GraphQLSubscription, GraphQLRequest, GraphQLResponse};
use actix_web_lab::respond::Html;
use futures_util::lock::Mutex;
use tonic::Request;


mod schema;
use schema::{PokerSchema, MutationRoot, QueryRoot, Storage, SubscriptionRoot};

use schema::deal::dealer_client::DealerClient;

async fn index_ws(
    schema: web::Data<PokerSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
}

/// GraphQL endpoint
#[route("/graphql", method = "GET", method = "POST")]
async fn graphql(schema: web::Data<PokerSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

/// GraphiQL playground UI
#[get("/graphiql")]
async fn graphql_playground() -> impl Responder {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/"),
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("GraphiQL IDE: http://localhost:8099");

    let deal_client: schema::DealService = Arc::new(Mutex::new(DealerClient::connect("http://127.0.0.1:5003").await?));
    let schema = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
    .data(Storage::default())
    .data(deal_client)
    //.data(subscriber)
    .finish();

    let _ = HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(schema.clone()))
        .service(graphql)
        .service(
            web::resource("/")
                .guard(guard::Get())
                .guard(guard::Header("upgrade", "websocket"))
                .to(index_ws),
        )
        .service(graphql_playground)
    })
    .bind("0.0.0.0:8099")?
    .run()
    .await?;

    Ok(())
}