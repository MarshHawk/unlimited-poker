use async_graphql::{Context, Object, Result, ID};
use async_trait::async_trait;
use deuces_rs::{builder::Dealer, GameDealer, RandomCardShuffler};
use float_ord::FloatOrd;
use rust_decimal::Decimal;
use std::mem;
use tonic::Request;
use uuid::Uuid;
use tokio::time::{timeout, Duration};
use mongodb::{Database};
use mongodb::bson::{doc, Document};
use mongodb::bson::to_bson;

use crate::bootstrap::schema::{
    deal::{HandRequest, HandResponse},
    simple_broker::SimpleBroker,
    DealEvent,
    DealService,
    HandEventPayload,
    HandToken,
    MutationType,
    TableToken, // Storage, TableToken,
    UserToken,
};

use super::model::{
    ActivePlayer, Cards, DealInput, Hand, Player, PlayerAction, PlayerEvent, PlayerInput,
    StreetEvent, StreetType,
};

use darkbird::{
    document::{self, RangeField},
    Options, Storage, StorageType,
};

pub struct MutationRoot;

#[async_trait]
#[cfg_attr(test, automock)]
pub trait GameMutations {
    async fn deal(&self, ctx: &Context<'_>, deal_input: DealInput) -> Result<ID>;
    async fn play_turn(
        &self,
        ctx: &Context<'_>,
        id: ID,
        player_id: ID,
        action: PlayerAction,
        amount: Decimal,
    ) -> Result<ID>;
}

#[Object]
#[async_trait]
impl GameMutations for MutationRoot {
    async fn deal(&self, ctx: &Context<'_>, deal_input: DealInput) -> Result<ID> {
        println!("MutationRoot::deal");

        // Get a handle to a collection in the database.

        //let user_token = ctx.data::<UserToken>().unwrap().0.clone();
        //let table_token = ctx.data::<TableToken>().unwrap().0.clone();

        //println!("user_token: {}", user_token);
        //println!("table_token: {}", table_token);

        // let deal_client = ctx.data_unchecked::<DealService>();
        // let req = Request::new(HandRequest {
        //     player_count: 3 as i32,
        // });

        let shuffler = RandomCardShuffler;
        let dealer: GameDealer<RandomCardShuffler> = GameDealer::new(shuffler);

        let deal_result = dealer.deal(3); //deal_client.lock().await.deal(req).await?.into_inner();
        let board = deal_result.board;
        println!("Deal board: {:?}", board);
        //let storage = ctx.data_unchecked::<Storage<String, Hand>>();
        let db = ctx.data_unchecked::<Database>();
        let collection = db.collection::<Document>("hands");
        
        // let mut hands = ctx.data_unchecked::<Storage::<String, Hand>>().lock().await;
        // println!("Deal hands: {:?}", hands);
        // let entry = hands.vacant_entry();
        // let id: ID = entry.key().into();
        let id = Uuid::new_v4();
        println!("Deal ID: {}", id.to_string());
        //let id: ID = Uuid::new_v4().into();
        let hand = Hand {
            id: id.to_string().into(),
            table_id: deal_input.table_id.into(),
            players: deal_input
                .players
                .iter()
                .enumerate()
                .map(|(i, p)| Player {
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
                        Decimal::new(10, 0)
                    } else {
                        Decimal::new(20, 0)
                    },
                    street_type: StreetType::Preflop,
                    action: PlayerAction::Bet,
                    player_id: p.id.to_owned(),
                    current_stack: if i == 0 {
                        p.stack - Decimal::new(10, 0)
                    } else {
                        p.stack - Decimal::new(20, 0)
                    },
                    current_pot: Decimal::new(30, 0),
                })
                .collect(),
            street_events: vec![StreetEvent {
                pot: Decimal::new(30, 0),
                current_active_players: sort_active_players(deal_input.players),
                street_type: StreetType::Preflop,
            }],
        };

        //println!("Deal requested, full hand: {:#?}", hand);

        //entry.insert(hand);
        //storage.insert(id.to_string().into(), hand).await.unwrap();
        // Insert some documents into the "mydb.books" collection.
        let hand_bson = to_bson(&hand)?;

        match hand_bson {
            mongodb::bson::Bson::Document(document) => {
                collection.insert_one(document, None).await?;
            }
            _ => return Err("Error converting hand to BSON document".into()),
        }

        SimpleBroker::publish(DealEvent {
            mutation_type: MutationType::Created,
            id: id.to_string().into(),
        });
        Ok(id.to_string().into())
    }

    async fn play_turn(
        &self,
        ctx: &Context<'_>,
        id: ID,
        player_id: ID,
        action: PlayerAction,
        amount: Decimal,
    ) -> Result<ID> {
        println!("MutationRoot::play_turn");

        let user_token = ctx.data::<UserToken>().unwrap().0.clone();
        let hand_token = ctx.data::<HandToken>().unwrap().0.clone();
        println!("user_token: {}", user_token);
        println!("hand_token: {}", hand_token);

        let db = ctx.data_unchecked::<Database>();
        let typed_collection = db.collection::<Hand>("hands");
        let hand_option = typed_collection.find_one(doc! { "id": hand_token }, None).await?;
        let mut hand = hand_option.ok_or_else(|| "No document found with the specified id".to_string())?;

        // get last player event:
        let last_player_event = hand.player_events.last().unwrap();

        // build current player event
        let player = hand
            .players
            .iter_mut()
            .find(|p| p.id == player_id)
            .ok_or("Player not found")?;
        player.stack -= amount;

        let current_player_event = PlayerEvent {
            amount: amount,
            street_type: last_player_event.street_type,
            action: action,
            player_id: player_id.clone(),
            current_stack: player.stack,
            current_pot: last_player_event.current_pot + amount,
        };

        hand.player_events.push(current_player_event);

        println!("current hand: {:#?}", hand);

        // Update active players on hand from last street event, set inactive if player event is fold:
        let mut active_players = hand
            .street_events
            .last()
            .unwrap()
            .current_active_players
            .clone();

        println!("active_players pre fold: {:#?}", active_players);

        let last_street_event: &mut StreetEvent = hand.street_events.iter_mut().last().unwrap();

        if action == PlayerAction::Bet {
            last_street_event.pot += amount;
        }

        let current_pot = last_street_event.pot;

        let mut cap = active_players.iter_mut();

        let active_player_count = cap.filter(|p| !p.is_inactive).count();

        let current_active_player = active_players
            .iter_mut()
            .find(|p| p.id == player_id)
            .ok_or("Player not found")?;
        current_active_player.bet += amount;
        let current_bet = current_active_player.bet;

        println!("active_player_count pre fold: {}", active_player_count);

        if action == PlayerAction::Fold {
            current_active_player.is_inactive = true;
        }

        let next_active_player_count = active_players.iter_mut().filter(|p| !p.is_inactive).count();
        println!(
            "active_player_count after fold: {}",
            next_active_player_count
        );

        let all_bets_equal = active_players.iter_mut().all(|p| p.bet == current_bet);

        let is_last_active_player = active_players.iter_mut().last().unwrap().id == player_id;

        let should_change_street = all_bets_equal && is_last_active_player;

        // what's the next street?
        let next_street_type = if should_change_street {
            match last_street_event.street_type {
                StreetType::Preflop => StreetType::Flop,
                StreetType::Flop => StreetType::Turn,
                StreetType::Turn => StreetType::River,
                StreetType::River => StreetType::Preflop,
            }
        } else {
            last_street_event.street_type
        };

        let game_over = next_active_player_count == 1
            || should_change_street && next_street_type == StreetType::Preflop;
        println!("game_over: {}", game_over);
        if game_over {
            // who won? e.g. who gets the pot?
            let mut winner_player_id: String = "TODO: find winner".to_string();
            if active_player_count == 1 {
                let winner = active_players.iter().find(|p| !p.is_inactive).unwrap();
                winner_player_id = winner.id.clone().to_string();
            } else {
                // find player in hand with the highest score:
                let winner_player = active_players
                    .iter()
                    .filter(|p| !p.is_inactive)
                    .max_by_key(|p| {
                        let player = hand.players.iter().find(|l| l.id == p.id).unwrap();
                        FloatOrd(player.score)
                    })
                    .ok_or("No winner found")?;

                winner_player_id = winner_player.id.clone().to_string();
            }
            let mut next_players = hand.players.clone();

            // map active_players stack to next_players stacks:
            for player in next_players.iter_mut() {
                let most_recent_player_event =
                    hand.player_events.iter().find(|p| p.player_id == player.id);
                // TODO: handle player not found
                player.stack = most_recent_player_event.unwrap().current_stack;
                if player.id.to_string() == winner_player_id {
                    player.stack += current_pot;
                }
            }

            let last_player = next_players.remove(0);
            next_players.push(last_player);

            let deal_input = DealInput {
                table_id: hand.table_id.clone().into(),
                players: next_players
                    .iter()
                    .map(|p| PlayerInput {
                        id: p.id.clone().into(),
                        stack: p.stack,
                    })
                    .collect(),
            };
            println!("before storage");
            let hand_bson = to_bson(&hand)?;
            match hand_bson {
                mongodb::bson::Bson::Document(document) => {
                    let update = doc! { "$set": document };
                    let upsert_result = typed_collection.find_one_and_update(doc! {"id": id.to_string()}, update, None).await?;
                    println!("Updated document with ID: {:?}", upsert_result);
                }
                _ => return Err("Error converting hand to BSON document".into()),
            }
            // storage.insert(hand_token, hand).await.unwrap();
            println!("Starting deal function");
            let result = self.deal(ctx, deal_input).await;
            println!("how to print line {:?}", result);
            println!("Finished deal function");
        } else {
            println!("not game over block");
            // move current_active_player to end of active_players array:
            let current_active_player = active_players.remove(0);
            active_players.push(current_active_player);

            // TODO: rotate to next active player
            println!("build next street event");
            let next_street_event = StreetEvent {
                pot: last_street_event.pot,
                current_active_players: active_players,
                street_type: next_street_type,
                //TODO: allowed actions
            };
            println!("push next street event");
            hand.street_events.push(next_street_event.clone());
            println!("build payload");
            let payload = HandEventPayload {
                mutation_type: MutationType::Updated,
                hand_id: id.clone(),
                street_event: Some(next_street_event),
                player_event: Some(hand.player_events.last().unwrap().clone()),
                cards: None, //TODO: cards
            };
            println!("before storage");
            let hand_bson = to_bson(&hand)?;
            match hand_bson {
                mongodb::bson::Bson::Document(document) => {
                    let update = doc! { "$set": document };
                    let upsert_result = typed_collection.find_one_and_update(doc! {"id": id.to_string()}, update, None).await?;
                    println!("Updated document with ID: {:?}", upsert_result);
                }
                _ => return Err("Error converting hand to BSON document".into()),
            }
            SimpleBroker::publish(payload.clone());
        }

        Ok(id.clone())
    }
}

fn build_active_players(players: Vec<PlayerInput>) -> Vec<ActivePlayer> {
    players
        .iter()
        .enumerate()
        .map(|(i, p)| match i {
            0 => ActivePlayer {
                id: p.id.clone(),
                bet: Decimal::new(10, 0),
                stack: p.stack - Decimal::new(10, 0),
                is_inactive: false,
                is_big_blind: false,
            },
            1 => ActivePlayer {
                id: p.id.clone(),
                bet: Decimal::new(20, 0),
                stack: p.stack - Decimal::new(20, 0),
                is_inactive: false,
                is_big_blind: true,
            },
            _ => ActivePlayer {
                id: p.id.clone(),
                bet: Decimal::new(0, 0),
                stack: p.stack,
                is_inactive: false,
                is_big_blind: false,
            },
        })
        .collect()
}

fn sort_active_players(players: Vec<PlayerInput>) -> Vec<ActivePlayer> {
    let mut active_players = build_active_players(players);
    if active_players.len() != 2 {
        let slice1: Vec<ActivePlayer> = active_players.split_off(2);
        let slice2: Vec<ActivePlayer> = active_players.drain(..2).collect();
        active_players.extend(slice1);
        active_players.extend(slice2);
    }
    active_players
}

#[cfg(test)]
use mockall::{automock, mock, predicate::*};

#[cfg(test)]
mod tests {
    use super::*;
    use async_graphql::{Context, Data, Enum, Object, Result, Schema, Subscription, ID};
    use mockall::automock;
    use mockall::mock;
    use mockall::predicate::*;
}
