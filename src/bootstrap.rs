use actix_web::{
    get, guard, http::header::HeaderMap, web, HttpRequest, HttpResponse, Responder, Result,
};
use actix_web_lab::respond::Html;
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Data, Schema,
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
//use serde_derive::{Deserialize, Serialize};
use mongodb::options::Credential;
use mongodb::{options::ClientOptions, Client};
use rdkafka::consumer::StreamConsumer;
use rdkafka::error::KafkaError;
use serde::{Deserialize, Serialize};
use std::env;
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::get_rdkafka_version;

use crate::bootstrap::schema::model::Hand;
pub mod schema {
    include!("./schema/mod.rs");
}
use schema::{
    mutation::MutationRoot, HandToken, PokerSchema, QueryRoot, SubscriptionRoot, TableToken,
    UserToken,
};

use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::{BaseConsumer, CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::error::KafkaResult;
use rdkafka::message::{Headers, Message};
use rdkafka::topic_partition_list::TopicPartitionList;

// A context can be used to change the behavior of producers and consumers by adding callbacks
// that will be executed by librdkafka.
// This particular context sets up custom callbacks to log rebalancing events.
struct ConsumerTestContext;

impl ClientContext for ConsumerTestContext {}

impl ConsumerContext for ConsumerTestContext {
    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        println!("Committing offsets: {:?}", result);
    }
}
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

pub fn kafka_seed() -> String {
    std::env::var("KAFKA_SEED").unwrap_or_else(|_| {
        let kafka_seed = "127.0.0.1:9092".to_string();
        ///warn!("using default kafka seed, {}", kafka_seed);
        kafka_seed
    })
}

fn new_consumer(brokers: String, topics: &[String]) -> Result<StreamConsumer, KafkaError> {
    let msg = topics.join(" ");
    // info!("subscribing to topics {}", msg);
    let stream_consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "test-group")
        .set("bootstrap.servers", &brokers)
        .set("auto.offset.reset", "latest")
        .set("enable.partition.eof", "true")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()?;
    let topics = topics
        .iter()
        .map(|topic| topic.as_str())
        .collect::<Vec<&str>>();
    stream_consumer.subscribe(topics.as_slice())?;
    Ok(stream_consumer)
}

pub async fn bootstrap_schema(
) -> Result<Schema<QueryRoot, MutationRoot, SubscriptionRoot>, Box<dyn std::error::Error>> {
    // mongo
    let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    let creds = Credential::builder()
        .username("root".to_string())
        .password("example".to_string())
        .build();
    client_options.credential = Some(creds.clone());
    println!(
        "Username: {}, Password: {}",
        creds.username.unwrap(),
        creds.password.unwrap()
    );
    let client = Client::with_options(client_options)?;
    let db = client.database("poker");

    //let topics = topics
    //    .iter()
    //    .map(|topic| topic.as_str())
    //    .collect::<Vec<&str>>();
    //stream_consumer.subscribe(topics.as_slice())?;
    //
    let topics = vec!["loony_topic".to_string()];
    let kafka_host = kafka_seed();
    let kafka_consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "test-group")
        .set("bootstrap.servers", &kafka_host)
        .set("auto.offset.reset", "latest")
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()?;

    let topics = topics
        .iter()
        .map(|topic| topic.as_str())
        .collect::<Vec<&str>>();
    kafka_consumer.subscribe(topics.as_slice())?;
    let kafka_producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", kafka_host)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    //let kafka_producer = ClientConfig::new()
    //    .set("bootstrap.servers", kafka_host)
    //    .set("produce.offset.report", "true")
    //    .set("message.timeout.ms", "5000")
    //    .create::<FutureProducer>();

    Ok(Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(db)
        .data(kafka_consumer)
        .data(kafka_producer)
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
