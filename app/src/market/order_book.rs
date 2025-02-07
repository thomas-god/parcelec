use std::{cmp::Ordering, collections::BinaryHeap};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::models::Direction;

#[derive(Debug, PartialEq, Clone)]
pub struct Trade {
    buyer: String,
    seller: String,
    volume: usize,
    price: isize,
    execution_time: DateTime<Utc>,
}

impl Trade {
    pub fn split(&self) -> (TradeLeg, TradeLeg) {
        let buy_trade_leg = TradeLeg {
            direction: Direction::Buy,
            volume: self.volume,
            price: self.price,
            execution_time: self.execution_time,
            owner: self.buyer.clone(),
        };
        let sell_trade_leg = TradeLeg {
            direction: Direction::Sell,
            volume: self.volume,
            price: self.price,
            execution_time: self.execution_time,
            owner: self.seller.clone(),
        };
        (buy_trade_leg, sell_trade_leg)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct TradeLeg {
    pub direction: Direction,
    pub volume: usize,
    pub price: isize,
    pub owner: String,
    pub execution_time: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
pub struct OrderRequest {
    pub direction: Direction,
    pub price: isize,
    pub volume: usize,
    pub owner: String,
}

#[derive(Debug)]
pub struct Order {
    pub id: String,
    pub direction: Direction,
    pub price: isize,
    pub volume: usize,
    pub timestamp: DateTime<Utc>,
    pub owner: String,
}

impl From<OrderRequest> for Order {
    fn from(request: OrderRequest) -> Self {
        Order {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            direction: request.direction,
            owner: request.owner,
            price: request.price,
            volume: request.volume,
        }
    }
}

#[derive(Debug)]
pub struct Bid(pub Order);

// Ord requires Eq + PartialOrd, that requires PartialEq
impl Eq for Bid {}

impl PartialOrd for Bid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Bid {
    fn eq(&self, other: &Self) -> bool {
        self.0.price == other.0.price && self.0.timestamp == other.0.timestamp
    }
}

impl Ord for Bid {
    fn cmp(&self, other: &Self) -> Ordering {
        match (
            self.0.price.cmp(&other.0.price),
            self.0.timestamp.cmp(&other.0.timestamp),
        ) {
            // Sort by ascending price, and descending timestamp
            (Ordering::Less, _) => Ordering::Less,
            (Ordering::Greater, _) => Ordering::Greater,
            (Ordering::Equal, Ordering::Less) => Ordering::Greater,
            (Ordering::Equal, Ordering::Greater) => Ordering::Less,
            (Ordering::Equal, Ordering::Equal) => Ordering::Equal,
        }
    }
}

#[derive(Debug)]
pub struct Offer(pub Order);

// Ord requires Eq + PartialOrd, that requires PartialEq
impl Eq for Offer {}

impl PartialOrd for Offer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Offer {
    fn eq(&self, other: &Self) -> bool {
        self.0.price == other.0.price && self.0.timestamp == other.0.timestamp
    }
}

impl Ord for Offer {
    fn cmp(&self, other: &Self) -> Ordering {
        match (
            self.0.price.cmp(&other.0.price),
            self.0.timestamp.cmp(&other.0.timestamp),
        ) {
            // Sort by descending price, and descending timestamp
            (Ordering::Less, _) => Ordering::Greater,
            (Ordering::Greater, _) => Ordering::Less,
            (Ordering::Equal, Ordering::Less) => Ordering::Greater,
            (Ordering::Equal, Ordering::Greater) => Ordering::Less,
            (Ordering::Equal, Ordering::Equal) => Ordering::Equal,
        }
    }
}

/// The `OrderBook` keeps tracks of the `Bid`s (order that want to BUY) and the `Offer`s (order that
/// want to SELL) for an associated delivery period.
///
/// One `Bid` can match an `Offer` if its price is greater of equal than the offer's price, and vice
/// versa. When they match, theyre are deleted from the `OrderBook` into a matching `Trade`.
///
/// Because the `OrderBook` internally maintains sorted lists of the bids and the offers, we only
/// need to check if an offer and a bid can match when trying to add a new one to the `OrderBook`.
/// Especially, when removing and order from the `OrderBook` we don't need to check if a trade is
/// possible, as it would have been found during the previous insertion in the `OrderBook`.
///
pub struct OrderBook {
    offers: BinaryHeap<Offer>,
    bids: BinaryHeap<Bid>,
    trades: Vec<Trade>,
}

pub struct OrderBookSnapshot<'a> {
    pub offers: &'a [Offer],
    pub bids: &'a [Bid],
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            offers: BinaryHeap::new(),
            bids: BinaryHeap::new(),
            trades: Vec::new(),
        }
    }

    pub fn register_order_request(&mut self, order_request: OrderRequest) -> Vec<Trade> {
        let order = Order::from(order_request);
        println!("Trying to register order: {order:?}");
        let trades = match order.direction {
            Direction::Buy => self.insert_bid(order),
            Direction::Sell => self.insert_offer(order),
        };
        for trade in trades.iter() {
            self.trades.push(trade.clone());
        }
        trades
    }

    pub fn remove_offer(&mut self, order_id: String) {
        self.bids.retain(|bid| bid.0.id != order_id);
        self.offers.retain(|offer| offer.0.id != order_id);
    }

    pub fn snapshot(&self) -> OrderBookSnapshot {
        OrderBookSnapshot {
            bids: self.bids.as_slice(),
            offers: self.offers.as_slice(),
        }
    }

    fn insert_bid(&mut self, order: Order) -> Vec<Trade> {
        let mut bid = Bid(order);
        let mut trades = Vec::<Trade>::new();
        while let Some(mut offer) = self.offers.pop() {
            match (
                bid.0.price.cmp(&offer.0.price),
                bid.0.volume.cmp(&offer.0.volume),
            ) {
                (Ordering::Less, _) => {
                    self.offers.push(offer);
                    break;
                }
                (Ordering::Equal, Ordering::Equal) | (Ordering::Greater, Ordering::Equal) => {
                    // Same volumes, both offer and bid are fully matched by the resulting trade
                    trades.push(Trade {
                        buyer: bid.0.owner.clone(),
                        seller: offer.0.owner.clone(),
                        price: offer.0.price,
                        volume: offer.0.volume,
                        execution_time: Utc::now(),
                    });
                    bid.0.volume = 0;
                    break;
                }
                (Ordering::Equal, Ordering::Greater) | (Ordering::Greater, Ordering::Greater) => {
                    // Some volume remains in the bid while the offer's volume is fully matched
                    trades.push(Trade {
                        buyer: bid.0.owner.clone(),
                        seller: offer.0.owner.clone(),
                        price: offer.0.price,
                        volume: offer.0.volume,
                        execution_time: Utc::now(),
                    });
                    bid.0.volume -= offer.0.volume;
                }
                (Ordering::Equal, Ordering::Less) | (Ordering::Greater, Ordering::Less) => {
                    // The bid has been fully matched, but the offer has some volume left
                    // and must be put back in the order book with its volume adjusted
                    trades.push(Trade {
                        buyer: bid.0.owner.clone(),
                        seller: offer.0.owner.clone(),
                        price: offer.0.price,
                        volume: bid.0.volume,
                        execution_time: Utc::now(),
                    });
                    offer.0.volume -= bid.0.volume;
                    bid.0.volume = 0;
                    self.offers.push(offer);
                    break;
                }
            }
        }
        if bid.0.volume > 0 {
            self.bids.push(bid);
        }
        trades
    }

    fn insert_offer(&mut self, order: Order) -> Vec<Trade> {
        let mut offer = Offer(order);
        let mut trades = Vec::<Trade>::new();
        while let Some(mut bid) = self.bids.pop() {
            println!("{offer:?}");
            println!("{bid:?}");
            match (
                offer.0.price.cmp(&bid.0.price),
                offer.0.volume.cmp(&bid.0.volume),
            ) {
                (Ordering::Greater, _) => {
                    self.bids.push(bid);
                    break;
                }
                (Ordering::Equal, Ordering::Equal) | (Ordering::Less, Ordering::Equal) => {
                    // Same volumes, both offer and bid are fully matched by the resulting trade
                    trades.push(Trade {
                        buyer: bid.0.owner.clone(),
                        seller: offer.0.owner.clone(),
                        price: bid.0.price,
                        volume: bid.0.volume,
                        execution_time: Utc::now(),
                    });
                    offer.0.volume = 0;
                    break;
                }
                (Ordering::Equal, Ordering::Greater) | (Ordering::Less, Ordering::Greater) => {
                    // Some volume remains in the offer while the bid's volume is fully matched
                    trades.push(Trade {
                        buyer: bid.0.owner.clone(),
                        seller: offer.0.owner.clone(),
                        price: bid.0.price,
                        volume: bid.0.volume,
                        execution_time: Utc::now(),
                    });
                    offer.0.volume -= bid.0.volume;
                }
                (Ordering::Equal, Ordering::Less) | (Ordering::Less, Ordering::Less) => {
                    // The offer has been fully matched, but the bid has some volume left
                    // and must be put back in the order book with its volume adjusted
                    trades.push(Trade {
                        buyer: bid.0.owner.clone(),
                        seller: offer.0.owner.clone(),
                        price: bid.0.price,
                        volume: offer.0.volume,
                        execution_time: Utc::now(),
                    });
                    bid.0.volume -= offer.0.volume;
                    offer.0.volume = 0;
                    self.bids.push(bid);
                    break;
                }
            }
        }
        if offer.0.volume > 0 {
            self.offers.push(offer);
        }
        trades
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {

    use crate::market::models::Direction;

    use super::{OrderBook, OrderRequest};

    fn build_order_request(
        direction: Direction,
        price: isize,
        volume: usize,
        owner: String,
    ) -> OrderRequest {
        OrderRequest {
            direction,
            price,
            volume,
            owner,
        }
    }

    #[test]
    fn test_register_order_empty_repository() {
        let mut repository = OrderBook::new();

        let order = build_order_request(Direction::Buy, 50_00, 10, "toto".to_string());

        let res = repository.register_order_request(order);
        assert!(res.is_empty());
    }

    #[test]
    fn test_register_two_orders_doesnt_match() {
        let mut repository = OrderBook::new();

        let buy_order = build_order_request(Direction::Buy, 50_00, 10, "toto".to_string());
        let sell_order = build_order_request(Direction::Sell, 50_01, 10, "tata".to_string());

        let res = repository.register_order_request(buy_order);
        assert!(res.is_empty());
        let res = repository.register_order_request(sell_order);
        assert!(res.is_empty());
    }

    #[test]
    fn test_match_2_orders_same_price_same_volume() {
        let mut repository = OrderBook::new();

        let buy_order = build_order_request(Direction::Buy, 50_00, 10, "toto".to_string());
        let sell_order = build_order_request(Direction::Sell, 50_00, 10, "tata".to_string());

        let res = repository.register_order_request(buy_order);
        assert!(res.is_empty(),);

        let res = repository.register_order_request(sell_order);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].buyer, "toto".to_string());
        assert_eq!(res[0].seller, "tata".to_string());
        assert_eq!(res[0].volume, 10);
        assert_eq!(res[0].price, 50_00);
    }

    #[test]
    fn test_match_2_orders_same_price_existing_order_lesser_volume() {
        let mut repository = OrderBook::new();

        let buy_order = build_order_request(Direction::Buy, 50_00, 5, "toto".to_string());
        let sell_order = build_order_request(Direction::Sell, 50_00, 10, "tata".to_string());

        let res = repository.register_order_request(buy_order);
        assert!(res.is_empty(),);

        let res = repository.register_order_request(sell_order);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].buyer, "toto".to_string());
        assert_eq!(res[0].seller, "tata".to_string());
        assert_eq!(res[0].volume, 5);
        assert_eq!(res[0].price, 50_00);
    }

    #[test]
    fn test_match_2_orders_same_price_existing_order_greater_volume() {
        let mut repository = OrderBook::new();

        let buy_order = build_order_request(Direction::Buy, 50_00, 15, "toto".to_string());
        let sell_order = build_order_request(Direction::Sell, 50_00, 10, "tata".to_string());

        let res = repository.register_order_request(buy_order);
        assert!(res.is_empty(),);

        let res = repository.register_order_request(sell_order);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].buyer, "toto".to_string());
        assert_eq!(res[0].seller, "tata".to_string());
        assert_eq!(res[0].volume, 10);
        assert_eq!(res[0].price, 50_00);
    }

    #[test]
    fn test_match_multiple_bids() {
        let mut order_book = OrderBook::new();

        let first_bid = build_order_request(Direction::Buy, 50_00, 10, "buyer_1".to_string());
        let second_bid = build_order_request(Direction::Buy, 49_00, 5, "buyer_2".to_string());

        order_book.register_order_request(first_bid);
        order_book.register_order_request(second_bid);

        let matching_offer = build_order_request(Direction::Sell, 49_00, 15, "seller".to_string());
        let res = order_book.register_order_request(matching_offer);
        assert_eq!(res.len(), 2);

        println!("{res:?}");
        assert_eq!(res[0].buyer, "buyer_1".to_string());
        assert_eq!(res[0].seller, "seller".to_string());
        assert_eq!(res[0].volume, 10);
        assert_eq!(res[0].price, 50_00);

        assert_eq!(res[1].buyer, "buyer_2".to_string());
        assert_eq!(res[1].seller, "seller".to_string());
        assert_eq!(res[1].volume, 5);
        assert_eq!(res[1].price, 49_00);
    }

    #[test]
    fn test_match_multiple_offers() {
        let mut order_book = OrderBook::new();

        let first_offer = build_order_request(Direction::Sell, 50_00, 10, "seller_1".to_string());
        let second_offer = build_order_request(Direction::Sell, 51_00, 5, "seller_2".to_string());

        order_book.register_order_request(first_offer);
        order_book.register_order_request(second_offer);

        let matching_bid = build_order_request(Direction::Buy, 51_00, 15, "buyer".to_string());
        let res = order_book.register_order_request(matching_bid);
        assert_eq!(res.len(), 2);

        println!("{res:?}");
        assert_eq!(res[0].buyer, "buyer".to_string());
        assert_eq!(res[0].seller, "seller_1".to_string());
        assert_eq!(res[0].volume, 10);
        assert_eq!(res[0].price, 50_00);

        assert_eq!(res[1].buyer, "buyer".to_string());
        assert_eq!(res[1].seller, "seller_2".to_string());
        assert_eq!(res[1].volume, 5);
        assert_eq!(res[1].price, 51_00);

        assert_eq!(order_book.offers.len(), 0);
        assert_eq!(order_book.bids.len(), 0);
    }

    #[test]
    fn test_no_match_dont_touch_existing_orders() {
        let mut order_book = OrderBook::new();

        let first_order = build_order_request(Direction::Sell, 51_00, 10, "seller".to_string());
        let second_order = build_order_request(Direction::Buy, 50_00, 5, "buyer".to_string());

        order_book.register_order_request(first_order);
        let trades = order_book.register_order_request(second_order);
        assert_eq!(trades.len(), 0);
        assert_eq!(order_book.bids.len(), 1);
        assert_eq!(order_book.offers.len(), 1);

        let third_order = build_order_request(Direction::Sell, 52_00, 10, "toto".to_string());
        let trades = order_book.register_order_request(third_order);
        assert_eq!(trades.len(), 0);
        assert_eq!(order_book.bids.len(), 1);
        assert_eq!(order_book.offers.len(), 2);
    }
}

#[cfg(test)]
mod test_remove_order {
    use super::{OrderBook, OrderRequest};

    #[test]
    fn test_remove_order() {
        let mut order_book = OrderBook::new();

        // Insert an order
        let first_order = OrderRequest {
            direction: super::Direction::Buy,
            volume: 10,
            price: 50_00,
            owner: "buyer".to_string(),
        };
        order_book.register_order_request(first_order);

        // Remove it from the order book
        let order_id = order_book.bids.peek().map(|bid| bid.0.id.clone()).unwrap();
        order_book.remove_offer(order_id);

        // Insert a matching offer, this shoudl not produce any trade
        let offer_that_would_have_matched = OrderRequest {
            direction: super::Direction::Sell,
            volume: 10,
            price: 50_00,
            owner: "seller".to_string(),
        };
        let trades = order_book.register_order_request(offer_that_would_have_matched);
        assert!(trades.is_empty());
    }
}

#[cfg(test)]
mod test_bid_and_offer {
    use std::cmp::Ordering;

    use chrono::Utc;
    use uuid::Uuid;

    use crate::market::{models::Direction, order_book::Offer};

    use super::{Bid, Order};

    #[test]
    fn test_bids_ordering() {
        fn build_bid(price: isize) -> Bid {
            Bid(Order {
                direction: Direction::Buy,
                price,
                owner: Uuid::new_v4().to_string(),
                volume: 10,
                timestamp: Utc::now(),
                id: Uuid::new_v4().to_string(),
            })
        }

        let bid = build_bid(50_00);
        assert_eq!(bid.cmp(&bid), Ordering::Equal);

        let more_expensive_bid = build_bid(50_01);
        assert_eq!(bid.cmp(&more_expensive_bid), Ordering::Less);

        let less_expensive_bid = build_bid(49_99);
        assert_eq!(bid.cmp(&less_expensive_bid), Ordering::Greater);

        let same_price_but_older_bid = build_bid(50_00);
        assert_eq!(bid.cmp(&same_price_but_older_bid), Ordering::Greater);
    }

    #[test]
    fn test_offers_ordering() {
        fn build_offer(price: isize) -> Offer {
            Offer(Order {
                direction: Direction::Sell,
                price,
                owner: Uuid::new_v4().to_string(),
                volume: 10,
                timestamp: Utc::now(),
                id: Uuid::new_v4().to_string(),
            })
        }

        let offer = build_offer(50_00);
        assert_eq!(offer.cmp(&offer), Ordering::Equal);

        let more_expensive_offer = build_offer(50_01);
        assert_eq!(offer.cmp(&more_expensive_offer), Ordering::Greater);

        let less_expensive_offer = build_offer(49_99);
        assert_eq!(offer.cmp(&less_expensive_offer), Ordering::Less);

        let same_price_but_older_offer = build_offer(50_00);
        assert_eq!(offer.cmp(&same_price_but_older_offer), Ordering::Greater);
    }
}

#[cfg(test)]
mod test_trade_leg {
    use chrono::Utc;

    use crate::market::{models::Direction, order_book::TradeLeg};

    use super::Trade;

    #[test]
    fn test_split_trade() {
        let trade = Trade {
            buyer: "buyer".to_string(),
            seller: "seller".to_string(),
            volume: 10,
            price: 50_00,
            execution_time: Utc::now(),
        };

        assert_eq!(
            trade.split(),
            (
                TradeLeg {
                    direction: Direction::Buy,
                    owner: "buyer".to_string(),
                    volume: 10,
                    price: 50_00,
                    execution_time: trade.execution_time
                },
                TradeLeg {
                    direction: Direction::Sell,
                    owner: "seller".to_string(),
                    volume: 10,
                    price: 50_00,
                    execution_time: trade.execution_time
                },
            )
        )
    }
}
