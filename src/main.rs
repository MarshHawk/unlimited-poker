use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer, Result, Responder, get, route};
use async_graphql::{http::{GraphQLPlaygroundConfig, playground_source}, Schema};
use async_graphql_actix_web::{GraphQLSubscription, GraphQLRequest, GraphQLResponse};
use actix_web_lab::respond::Html;
// Add the books crate as a dependency

mod books;
use books::{BooksSchema, MutationRoot, QueryRoot, Storage, SubscriptionRoot};

async fn index_ws(
    schema: web::Data<BooksSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
}

/// GraphQL endpoint
#[route("/graphql", method = "GET", method = "POST")]
async fn graphql(schema: web::Data<BooksSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

/// GraphiQL playground UI
#[get("/graphiql")]
async fn graphql_playground() -> impl Responder {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/"),
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("GraphiQL IDE: http://localhost:8099");

    let schema = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
    .data(Storage::default())
    .finish();

    HttpServer::new(move || {
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
    .await
}