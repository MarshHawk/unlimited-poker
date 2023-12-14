use std::{sync::Arc, time::Duration};

use async_graphql::{Context, Enum, Object, Result, Schema, Subscription, ID};
use futures_util::{lock::Mutex, Stream, StreamExt};
use rust_decimal::Decimal;
use slab::Slab;

use tonic::Request;
use uuid::Uuid;

mod model;
use model::{Cards, DealInput, Hand, PlayerAction, PlayerEvent, StreetEvent, StreetType, ActivePlayer};
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
    async fn deal(&self, ctx: &Context<'_>, deal_input: DealInput) -> Result<ID> {
        let deal_client = ctx.data_unchecked::<DealService>();
        let req = Request::new(HandRequest {
            player_count: 3 as i32,
        });

        let deal_result: HandResponse = deal_client.lock().await.deal(req).await?.into_inner();
        let board = deal_result.board.unwrap();
        let mut hands = ctx.data_unchecked::<Storage>().lock().await;
        let entry = hands.vacant_entry();
        let id: ID = entry.key().into();
        //let id: ID = Uuid::new_v4().into();
        let hand = Hand {
            id: id.clone(),
            table_id: deal_input.table_id.into(),
            players: deal_input
                .players
                .iter()
                .enumerate()
                .map(|(i, p)| model::Player {
                    id: p.id.to_owned(),
                    stack: p.stack,
                    score: deal_result.hands[i].score,
                    cards: deal_result.hands[i].cards.clone(),
                    description: deal_result.hands[i].description.clone(),
                })
                .collect(),
            cards: Cards {
                flop: board.flop,
                turn: board.turn,
                river: board.river,
            },
            player_events: deal_input
                .players
                .iter()
                .enumerate()
                .filter(|(i, _)| [0, 1].contains(i))
                .map(|(i, p)| PlayerEvent {
                    amount: if i == 0 {
                        Decimal::new(10, 2)
                    } else {
                        Decimal::new(20, 2)
                    },
                    street_type: StreetType::Preflop,
                    action: PlayerAction::Bet,
                    player_id: p.id.to_owned(),
                    current_stack: if i == 0 {
                        p.stack - Decimal::new(10, 2)
                    } else {
                        p.stack - Decimal::new(20, 2)
                    },
                    current_pot: Decimal::new(30, 2),
                })
                .collect(),
            street_events: vec![StreetEvent {
                pot: Decimal::new(30, 2),
                should_increment_cycle: false,
                cycle_count: 0,
                current_active_players: deal_input
                    .players
                    .iter()
                    .enumerate()
                    .map(|(i, p)| ActivePlayer {
                        id: p.id.to_owned(),
                        stack: p.stack,
                        bet: if i == 0 {
                                Decimal::new(10, 2)
                            } else if i == 1 {
                                Decimal::new(20, 2)
                            }
                            else {
                                Decimal::new(0, 2)
                            },
                        is_inactive: false,
                    })
                    .collect(),
                street_type: StreetType::Preflop,
            }],
        };

        println!("Requested deal: {:#?}", hand);
        

        entry.insert(hand);

        SimpleBroker::publish(DealEvent {
            mutation_type: MutationType::Created,
            id: id.clone(),
        });
        Ok(id.clone())
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
