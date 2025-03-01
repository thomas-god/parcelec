use std::{fmt::Debug, future::Future};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, ser::SerializeStruct};
use tokio::sync::watch;

use order_book::{Bid, Offer, OrderRequest, Trade, TradeLeg};

use crate::{forecast::ForecastLevel, game::delivery_period::DeliveryPeriodId, player::PlayerId};

pub mod actor;
pub mod order_book;
pub mod service;

pub use actor::{MarketActor, MarketMessage};
pub use service::MarketService;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Direction {
    Buy,
    Sell,
}

/// [Market] is the public API for the market domain of Parcelec. The market domain is
/// responsible for receiving and matching orders from players, as long a providing on update and
/// on demand snapshots of the order book (the list of currents orders).
pub trait Market: Clone + Send + Sync + 'static {
    /// Open the market, allowing players to send orders to the market.
    fn open_market(&self, delivery_period: DeliveryPeriodId) -> impl Future<Output = ()> + Send;

    /// Close the market, deleting outstanding orders and returning a list of the trades from the
    /// closing delivery period. Trying to close a delivery period already closed will have no side
    /// effect and will return the trade list for the closed delivery period.
    fn close_market(
        &self,
        delivery_period: DeliveryPeriodId,
    ) -> impl Future<Output = Vec<Trade>> + Send;

    /// Register a player to the market, sending an initial order book snapshot and a list of the
    /// player's trade for the current delivery period. Player can register even if the market is
    /// closed.
    fn get_market_snapshot(
        &self,
        player: PlayerId,
    ) -> impl Future<Output = (Vec<TradeLeg>, OBS, Vec<MarketForecast>)> + Send;

    /// Post a new order for the current delivery period. If the market is closed the request is
    /// ignored.
    fn new_order(&self, request: OrderRequest) -> impl Future<Output = ()> + Send;

    /// Delete an order from the market. Silently fails if the order does not exist or if the market
    /// is closed.
    fn delete_order(&self, order_id: String) -> impl Future<Output = ()> + Send;

    /// Allow a player (real or a bot) to register a forecast for a given delivery period. A
    /// forecast notifies the rest of the market participants of the player's intent to buy or sell
    /// an amount of energy at an optional price. This is not binding, and the player is not
    /// enforced to post matching order(s) during the actual delivery period.
    fn register_forecast(&self, forecast: MarketForecast) -> impl Future<Output = ()> + Send;
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OrderRepr {
    pub order_id: String,
    pub direction: Direction,
    pub volume: usize,
    pub price: isize,
    pub created_at: DateTime<Utc>,
    pub owned: bool,
}

impl OrderRepr {
    fn from_offer(offer: &Offer, player_id: Option<&PlayerId>) -> Self {
        OrderRepr {
            order_id: offer.0.id.clone(),
            direction: offer.0.direction.clone(),
            price: offer.0.price,
            volume: offer.0.volume,
            created_at: offer.0.timestamp,
            owned: player_id.map(|id| *id == offer.0.owner).unwrap_or(false),
        }
    }
    fn from_bid(bid: &Bid, player_id: Option<&PlayerId>) -> Self {
        OrderRepr {
            order_id: bid.0.id.clone(),
            direction: bid.0.direction.clone(),
            price: bid.0.price,
            volume: bid.0.volume,
            created_at: bid.0.timestamp,
            owned: player_id.map(|id| *id == bid.0.owner).unwrap_or(false),
        }
    }
}

#[derive(Debug)]
pub struct OBS {
    pub bids: Vec<OrderRepr>,
    pub offers: Vec<OrderRepr>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MarketState {
    Open,
    Closed,
}

impl Serialize for MarketState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("MarketState", 2)?;
        state.serialize_field("type", "MarketState")?;
        state.serialize_field(
            "state",
            match self {
                Self::Closed => "Closed",
                Self::Open => "Open",
            },
        )?;
        state.end()
    }
}

#[derive(Debug, Clone)]
pub struct MarketContext<MS: Market> {
    pub service: MS,
    pub state_rx: watch::Receiver<MarketState>,
}

#[derive(Debug, Serialize, Clone)]
pub struct MarketForecast {
    pub issuer: PlayerId,
    pub period: DeliveryPeriodId,
    pub direction: Direction,
    pub volume: ForecastLevel,
    pub price: Option<ForecastLevel>,
}

#[cfg(test)]
mod test_order_repr {
    use chrono::Utc;
    use uuid::Uuid;

    use crate::{
        market::{OrderRepr, order_book::Bid},
        player::PlayerId,
    };

    use super::{
        Direction,
        order_book::{Offer, Order},
    };

    #[test]
    fn test_order_repr_ownership_from_offer() {
        let offer = Offer(Order {
            direction: Direction::Sell,
            id: Uuid::new_v4().to_string(),
            owner: PlayerId::from("toto"),
            price: 10_00,
            timestamp: Utc::now(),
            volume: 100,
        });

        assert!(!OrderRepr::from_offer(&offer, None).owned);
        assert!(OrderRepr::from_offer(&offer, Some(&PlayerId::from("toto"))).owned);
        assert!(!OrderRepr::from_offer(&offer, Some(&PlayerId::from("not_toto"))).owned);
    }

    #[test]
    fn test_order_repr_ownership_from_bid() {
        let bid = Bid(Order {
            direction: Direction::Buy,
            id: Uuid::new_v4().to_string(),
            owner: PlayerId::from("toto"),
            price: 10_00,
            timestamp: Utc::now(),
            volume: 100,
        });

        assert!(!OrderRepr::from_bid(&bid, None).owned);
        assert!(OrderRepr::from_bid(&bid, Some(&PlayerId::from("toto"))).owned);
        assert!(!OrderRepr::from_bid(&bid, Some(&PlayerId::from("not_toto"))).owned);
    }
}

#[cfg(test)]
mod test_market_state {
    use crate::market::MarketState;

    #[test]
    fn test_game_state_serialize() {
        assert_eq!(
            serde_json::to_string(&MarketState::Open).unwrap(),
            "{\"type\":\"MarketState\",\"state\":\"Open\"}".to_string()
        );
        assert_eq!(
            serde_json::to_string(&MarketState::Closed).unwrap(),
            "{\"type\":\"MarketState\",\"state\":\"Closed\"}".to_string()
        );
    }
}
