use async_graphql::*;
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq)]
pub struct Hand {
    pub id: ID,
    pub table_id: ID,
    pub players: Vec<Player>,
    pub cards: Cards,
    pub player_events: Vec<PlayerEvent>,
    pub street_events: Vec<StreetEvent>,
}

#[Object]
impl Hand {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn table_id(&self) -> &ID {
        &self.table_id
    }

    async fn players(&self) -> &[Player] {
        &self.players
    }

    async fn cards(&self) -> &Cards {
        &self.cards
    }

    async fn player_events(&self) -> &[PlayerEvent] {
        &self.player_events
    }

    async fn street_events(&self) -> &[StreetEvent] {
        &self.street_events
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cards {
    pub flop: Vec<String>,
    pub turn: String,
    pub river: String,
}

#[Object]
impl Cards {
    async fn flop(&self) -> &[String] {
        &self.flop
    }

    async fn turn(&self) -> &str {
        &self.turn
    }

    async fn river(&self) -> &str {
        &self.river
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerEvent {
    pub player_id: ID,
    pub action: PlayerAction,
    pub amount: Decimal,
    pub street_type: StreetType,
    pub current_stack: Option<Decimal>,
    pub current_pot: Option<Decimal>,
}

#[Object]
impl PlayerEvent {
    async fn player_id(&self) -> &ID {
        &self.player_id
    }

    async fn action(&self) -> PlayerAction {
        self.action
    }

    async fn amount(&self) -> Decimal {
        self.amount
    }

    async fn street_type(&self) -> StreetType {
        self.street_type
    }

    async fn current_stack(&self) -> Option<Decimal> {
        self.current_stack
    }

    async fn current_pot(&self) -> Option<Decimal> {
        self.current_pot
    }
}

#[derive(Debug, Enum, Eq, PartialEq, Copy, Clone)]
pub enum PlayerAction {
    Bet,
    Check,
    Fold,
}

#[derive(Debug, Enum, Eq, PartialEq, Copy, Clone)]
pub enum StreetType {
    Preflop,
    Flop,
    Turn,
    River,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StreetEvent {
    pub street_type: StreetType,
    pub current_active_players: Vec<ActivePlayer>,
    pub pot: f64,
    pub cycle_count: u32,
    pub should_increment_cycle: bool,
}

#[Object]
impl StreetEvent {
    async fn street_type(&self) -> StreetType {
        self.street_type
    }

    async fn current_active_players(&self) -> &[ActivePlayer] {
        &self.current_active_players
    }

    async fn pot(&self) -> f64 {
        self.pot
    }

    async fn cycle_count(&self) -> u32 {
        self.cycle_count
    }

    async fn should_increment_cycle(&self) -> bool {
        self.should_increment_cycle
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActivePlayer {
    pub id: String,
    pub bet: f64,
    pub stack: f64,
    pub is_inactive: Option<bool>,
}

#[Object]
impl ActivePlayer {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn bet(&self) -> f64 {
        self.bet
    }

    async fn stack(&self) -> f64 {
        self.stack
    }

    async fn is_inactive(&self) -> Option<bool> {
        self.is_inactive
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DealInput {
    pub players: Vec<Player>,
    pub table_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub id: String,
    pub stack: Option<f64>,
    pub cards: Vec<String>,
    pub score: f64,
    pub description: String,
}

#[Object]
impl Player {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn stack(&self) -> Option<f64> {
        self.stack
    }

    async fn cards(&self) -> &[String] {
        &self.cards
    }

    async fn score(&self) -> f64 {
        self.score
    }

    async fn description(&self) -> &str {
        &self.description
    }
}