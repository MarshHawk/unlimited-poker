use std::{sync::Arc, time::Duration};

use async_graphql::{Context, Object, Result, Enum, Schema, Subscription, ID};
use futures_util::{lock::Mutex, Stream, StreamExt};
use slab::Slab;

use tonic::Request;
use uuid::Uuid;

mod model;
use model::{Hand, Cards};
mod simple_broker;
use simple_broker::SimpleBroker;
pub mod deal {
    include!("../deal_app.rs");
}
use deal::dealer_client::DealerClient;

use deal::{HandRequest, HandResponse};

pub type PokerSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

pub type Storage = Arc<Mutex<Slab<Hand>>>;

pub type DealService = Arc<Mutex<DealerClient<tonic::transport::Channel>>>;

#[derive(Enum, Eq, PartialEq, Copy, Clone)]
enum MutationType {
    Created,
    Updated,
}

#[derive(Clone)]
struct DealEvent {
    mutation_type: MutationType,
    id: ID,
}

#[Object]
impl DealEvent {
    async fn mutation_type(&self) -> MutationType {
        self.mutation_type
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


pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hands(&self, ctx: &Context<'_>) -> Vec<Hand> {
        let hands = ctx.data_unchecked::<Storage>().lock().await;
        hands.iter().map(|(_, book)| book).cloned().collect()
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn deal(&self, ctx: &Context<'_>, player_ids: Vec<ID>) -> Result<ID> {
        let deal_client = ctx.data_unchecked::<DealService>();
        let req = Request::new(HandRequest {
            player_count: 3 as i32,
        });
        ////println!("deal request: {:#?}", ctx.data_unchecked);
        ////let deal_client = ctx.data_unchecked::<DealService>().clone();
        let hand: HandResponse = deal_client.lock().await.deal(req).await?.into_inner();
        println!("Requested deal: {:#?}", hand);
        let mut hands = ctx.data_unchecked::<Storage>().lock().await;
        let entry = hands.vacant_entry();
        let id: ID = entry.key().into();
        let hand = Hand {
            id: id.clone(),
            table_id: String::from("table_id").into(),
            players: vec![],
            cards: Cards {
                flop: vec![],
                turn: String::from(""),
                river: String::from(""),
            },
            player_events: vec![],
            street_events: vec![],
        };

        entry.insert(hand);
        SimpleBroker::publish(DealEvent {
            mutation_type: MutationType::Created,
            id: id.clone(),
        });
        Ok(id)
    }
}

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn deal(&self, mutation_type: Option<MutationType>) -> impl Stream<Item = DealEvent> {
        SimpleBroker::<DealEvent>::subscribe().filter(move |event| {
            let res = if let Some(mutation_type) = mutation_type {
                event.mutation_type == mutation_type
            } else {
                true
            };
            async move { res }
        })
    }
}