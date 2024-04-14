use std::{sync::Arc, time::Duration};

use async_graphql::{Context, Enum, Object, Result, Schema, Subscription, ID};
use float_ord::FloatOrd;
use futures::stream::TryStreamExt;
use futures::StreamExt;
use futures_util::{lock::Mutex, Stream};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rust_decimal::Decimal;
use slab::Slab;
use rdkafka::message::{Headers, Message};

use tonic::Request;
use uuid::Uuid;

use mongodb::bson::to_bson;
use mongodb::bson::{doc, Document};
use mongodb::Database;

pub mod model;
use model::{
    ActivePlayer, Cards, DealInput, Hand, PlayerAction, PlayerEvent, PlayerInput, StreetEvent,
    StreetType,
};
mod simple_broker;
use simple_broker::SimpleBroker;

pub mod mutation;
use mutation::MutationRoot;
pub mod deal {
    include!("../deal_app.rs");
}
use deal::dealer_client::DealerClient;

use deal::{HandRequest, HandResponse};

pub type PokerSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

pub type DealService = Arc<Mutex<DealerClient<tonic::transport::Channel>>>;

#[derive(Debug)]
pub struct UserToken(pub String);

pub struct TableToken(pub String);

pub struct HandToken(pub String);

#[derive(Enum, Eq, PartialEq, Copy, Clone)]
pub enum MutationType {
    Created,
    Updated,
}

#[derive(Clone)]
struct HandEvent {
    mutation_type: MutationType,
    id: ID,
}

#[Object]
impl HandEvent {
    async fn mutation_type(&self) -> MutationType {
        MutationType::Created
    }

    async fn id(&self) -> &ID {
        &self.id
    }
}

#[derive(Clone)]
struct DealEvent {
    mutation_type: MutationType,
    id: ID,
}

#[Object]
impl DealEvent {
    async fn mutation_type(&self) -> MutationType {
        MutationType::Created
    }

    async fn id(&self) -> &ID {
        &self.id
    }

    async fn deal(&self, ctx: &Context<'_>) -> Result<Hand> {
        let db = ctx.data_unchecked::<Database>();
        let typed_collection = db.collection::<Hand>("hands");
        let id = self.id.parse::<String>()?;
        println!("id for mongo: {:#?}", id);
        let hand_option = typed_collection.find_one(doc! { "id": id }, None).await?;
        let hand =
            hand_option.ok_or_else(|| "No document found with the specified id".to_string())?;
        println!("here is the hand: {:?}", hand);
        //let mut fcursor = typed_collection.find(doc! { "id": id }, None).await?;
        //let fhand = fcursor.try_next().await?;
        //let hand_option = cursor.try_next().await?;
        //let hand = hand_option.ok_or("No document found with the specified id")?;

        Ok(hand)
    }
}

#[derive(Clone)]
pub struct HandEventPayload {
    mutation_type: MutationType,
    hand_id: ID,
    street_event: Option<StreetEvent>,
    player_event: Option<PlayerEvent>,
    cards: Option<Cards>,
}

#[Object]
impl HandEventPayload {
    async fn mutation_type(&self) -> MutationType {
        MutationType::Updated
    }

    async fn hand_id(&self) -> &ID {
        &self.hand_id
    }

    async fn street_event(&self) -> &Option<StreetEvent> {
        &self.street_event
    }

    async fn player_event(&self) -> &Option<PlayerEvent> {
        &self.player_event
    }

    async fn cards(&self) -> &Option<Cards> {
        &self.cards
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hands(&self, ctx: &Context<'_>) -> Vec<Hand> {
        Vec::new()
    }
}

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn deal(
        &self,
        ctx: &Context<'_>,
        mutation_type: Option<MutationType>,
    ) -> impl Stream<Item = DealEvent> {
        println!("SubscriptionRoot::deal");

        let user_token = ctx.data::<UserToken>().unwrap().0.clone();
        let table_token = ctx.data::<TableToken>().unwrap().0.clone();

        println!("user_token: {}", user_token);
        println!("table_token: {}", table_token);

        SimpleBroker::<DealEvent>::subscribe().filter(move |event| {
            let res = if let Some(mutation_type) = mutation_type {
                event.mutation_type == mutation_type
            } else {
                true
            };
            async move { res }
        })
    }

    async fn kafka_test<'a>(&self, ctx: &'a Context<'a>) -> impl Stream<Item = String> + 'a {
        let consumer: &'a StreamConsumer = ctx.data_unchecked::<StreamConsumer>();
        let message_stream = consumer.stream();
    
        message_stream.filter_map(|message_result| {
            async move {
                match message_result {
                    Ok(message) => {
                        message.payload_view::<str>().unwrap_or(Ok("")).ok().map(|s| s.to_string())
                    }
                    Err(e) => {
                        eprintln!("Error receiving message: {}", e);
                        None
                    }
                }
            }
        })
    }

    async fn hand_event(
        &self,
        ctx: &Context<'_>,
        mutation_type: Option<MutationType>,
    ) -> impl Stream<Item = HandEventPayload> {
        println!("SubscriptionRoot::hand_event");

        let user_token = ctx.data::<UserToken>().unwrap().0.clone();
        let table_token = ctx.data::<TableToken>().unwrap().0.clone();
        let hand_token = ctx.data::<HandToken>().unwrap().0.clone();

        println!("user_token: {}", user_token);
        println!("table_token: {}", table_token);
        println!("hand_token: {}", hand_token);

        SimpleBroker::<HandEventPayload>::subscribe().filter(move |event| {
            let res = if let Some(mutation_type) = mutation_type {
                event.mutation_type == mutation_type
            } else {
                true
            };
            async move { res }
        })
    }

    // HandEventPayload
}
