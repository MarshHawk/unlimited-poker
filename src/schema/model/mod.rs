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
    pub current_stack: Decimal,
    pub current_pot: Decimal,
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

    async fn current_stack(&self) -> Decimal {
        self.current_stack
    }

    async fn current_pot(&self) -> Decimal {
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
    pub pot: Decimal
}

#[Object]
impl StreetEvent {
    async fn street_type(&self) -> StreetType {
        self.street_type
    }

    async fn current_active_players(&self) -> &[ActivePlayer] {
        &self.current_active_players
    }

    async fn pot(&self) -> Decimal {
        self.pot
    }

}

#[derive(Clone, Debug, PartialEq)]
pub struct ActivePlayer {
    pub id: ID,
    pub bet: Decimal,
    pub stack: Decimal,
    pub is_inactive: bool,
    pub is_big_blind: bool,
}

#[Object]
impl ActivePlayer {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn bet(&self) -> Decimal {
        self.bet
    }

    async fn stack(&self) -> Decimal {
        self.stack
    }

    async fn is_inactive(&self) -> bool {
        self.is_inactive
    }

    async fn is_big_blind(&self) -> bool {
        self.is_big_blind
    }
}

#[derive(Clone, Debug, PartialEq, InputObject)]
pub struct DealInput {
    pub players: Vec<PlayerInput>,
    pub table_id: ID,
}

#[derive(Clone, Debug, PartialEq, InputObject)]
pub struct Player {
    pub id: ID,
    pub stack: Decimal,
    pub cards: Vec<String>,
    pub score: f64,
    pub description: String,
}

#[Object]
impl Player {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn stack(&self) -> Decimal {
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

#[derive(Clone, Debug, PartialEq, InputObject)]
pub struct PlayerInput {
    pub id: ID,
    pub stack: Decimal
}