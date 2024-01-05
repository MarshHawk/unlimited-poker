use std::{sync::Arc, time::Duration};

use async_graphql::{Context, Enum, Object, Result, Schema, Subscription, ID};
use futures_util::{lock::Mutex, Stream, StreamExt};
use rust_decimal::Decimal;
use slab::Slab;
use float_ord::FloatOrd;

use tonic::Request;
use uuid::Uuid;

mod model;
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

pub type Storage = Arc<Mutex<Slab<Hand>>>;

pub type DealService = Arc<Mutex<DealerClient<tonic::transport::Channel>>>;

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

    async fn deal(&self, ctx: &Context<'_>) -> Result<Option<Hand>> {
        let hands = ctx.data_unchecked::<Storage>().lock().await;
        let id = self.id.parse::<usize>()?;
        Ok(hands.get(id).cloned())
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
        let hands = ctx.data_unchecked::<Storage>().lock().await;
        hands.iter().map(|(_, book)| book).cloned().collect()
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
