use std::{cmp::Ordering, collections::BinaryHeap, time::Instant};

use chrono::{DateTime, Utc};

use crate::market::Direction;

#[derive(Debug, PartialEq)]
pub struct Trade {
    buyer: String,
    seller: String,
    volume: usize,
    price: usize,
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

#[derive(Debug, PartialEq)]
pub struct TradeLeg {
    direction: Direction,
    volume: usize,
    price: usize,
    owner: String,
    execution_time: DateTime<Utc>,
}

pub struct Order {
    direction: Direction,
    price: usize,
    volume: usize,
    timestamp: Instant,
    owner: String,
}

pub struct Bid(Order);

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
            // Sort by descending price, and descending timestamp
            (Ordering::Less, _) => Ordering::Greater,
            (Ordering::Greater, _) => Ordering::Less,
            (Ordering::Equal, Ordering::Less) => Ordering::Greater,
            (Ordering::Equal, Ordering::Greater) => Ordering::Less,
            (Ordering::Equal, Ordering::Equal) => Ordering::Equal,
        }
    }
}

pub struct Offer(Order);

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
            // Sort by acending price, and descending timestamp
            (Ordering::Less, _) => Ordering::Less,
            (Ordering::Greater, _) => Ordering::Greater,
            (Ordering::Equal, Ordering::Less) => Ordering::Greater,
            (Ordering::Equal, Ordering::Greater) => Ordering::Less,
            (Ordering::Equal, Ordering::Equal) => Ordering::Equal,
        }
    }
}

pub struct OrderBook {
    offers: BinaryHeap<Offer>,
    bids: BinaryHeap<Bid>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            offers: BinaryHeap::new(),
            bids: BinaryHeap::new(),
        }
    }

    pub fn register_offer(&mut self, order: Order) -> Vec<Trade> {
        match order.direction {
            Direction::Buy => self.insert_bid(order),
            Direction::Sell => self.insert_offer(order),
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
                (Ordering::Less, _) => break,
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
            match (
                offer.0.price.cmp(&bid.0.price),
                offer.0.volume.cmp(&bid.0.volume),
            ) {
                (Ordering::Greater, _) => break,
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
    use std::time::Instant;

    use crate::{market::Direction, order_book::Order};

    use super::OrderBook;

    fn build_order(direction: Direction, price: usize, volume: usize, owner: String) -> Order {
        Order {
            direction,
            price,
            volume,
            timestamp: Instant::now(),
            owner,
        }
    }

    #[test]
    fn test_register_order_empty_repository() {
        let mut repository = OrderBook::new();

        let order = build_order(Direction::Buy, 50_00, 10, "toto".to_string());

        let res = repository.register_offer(order);
        assert!(res.is_empty());
    }

    #[test]
    fn test_register_two_orders_doesnt_match() {
        let mut repository = OrderBook::new();

        let buy_order = build_order(Direction::Buy, 50_00, 10, "toto".to_string());
        let sell_order = build_order(Direction::Sell, 50_01, 10, "tata".to_string());

        let res = repository.register_offer(buy_order);
        assert!(res.is_empty());
        let res = repository.register_offer(sell_order);
        assert!(res.is_empty());
    }

    #[test]
    fn test_match_2_orders_same_price_same_volume() {
        let mut repository = OrderBook::new();

        let buy_order = build_order(Direction::Buy, 50_00, 10, "toto".to_string());
        let sell_order = build_order(Direction::Sell, 50_00, 10, "tata".to_string());

        let res = repository.register_offer(buy_order);
        assert!(res.is_empty(),);

        let res = repository.register_offer(sell_order);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].buyer, "toto".to_string());
        assert_eq!(res[0].seller, "tata".to_string());
        assert_eq!(res[0].volume, 10);
        assert_eq!(res[0].price, 50_00);
    }

    #[test]
    fn test_match_2_orders_same_price_existing_order_lesser_volume() {
        let mut repository = OrderBook::new();

        let buy_order = build_order(Direction::Buy, 50_00, 5, "toto".to_string());
        let sell_order = build_order(Direction::Sell, 50_00, 10, "tata".to_string());

        let res = repository.register_offer(buy_order);
        assert!(res.is_empty(),);

        let res = repository.register_offer(sell_order);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].buyer, "toto".to_string());
        assert_eq!(res[0].seller, "tata".to_string());
        assert_eq!(res[0].volume, 5);
        assert_eq!(res[0].price, 50_00);
    }

    #[test]
    fn test_match_2_orders_same_price_existing_order_greater_volume() {
        let mut repository = OrderBook::new();

        let buy_order = build_order(Direction::Buy, 50_00, 15, "toto".to_string());
        let sell_order = build_order(Direction::Sell, 50_00, 10, "tata".to_string());

        let res = repository.register_offer(buy_order);
        assert!(res.is_empty(),);

        let res = repository.register_offer(sell_order);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].buyer, "toto".to_string());
        assert_eq!(res[0].seller, "tata".to_string());
        assert_eq!(res[0].volume, 10);
        assert_eq!(res[0].price, 50_00);
    }

    #[test]
    fn test_match_multiple_bids() {
        let mut order_book = OrderBook::new();

        let first_bid = build_order(Direction::Buy, 50_00, 10, "buyer_1".to_string());
        let second_bid = build_order(Direction::Buy, 49_00, 5, "buyer_2".to_string());

        order_book.register_offer(first_bid);
        order_book.register_offer(second_bid);

        let matching_offer = build_order(Direction::Sell, 49_00, 15, "seller".to_string());
        let res = order_book.register_offer(matching_offer);
        assert_eq!(res.len(), 2);

        println!("{res:?}");
        assert_eq!(res[1].buyer, "buyer_1".to_string());
        assert_eq!(res[1].seller, "seller".to_string());
        assert_eq!(res[1].volume, 10);
        assert_eq!(res[1].price, 50_00);

        assert_eq!(res[0].buyer, "buyer_2".to_string());
        assert_eq!(res[0].seller, "seller".to_string());
        assert_eq!(res[0].volume, 5);
        assert_eq!(res[0].price, 49_00);
    }

    #[test]
    fn test_match_multiple_offers() {
        let mut order_book = OrderBook::new();

        let first_offer = build_order(Direction::Sell, 50_00, 10, "seller_1".to_string());
        let second_offer = build_order(Direction::Sell, 51_00, 5, "seller_2".to_string());

        order_book.register_offer(first_offer);
        order_book.register_offer(second_offer);

        let matching_bid = build_order(Direction::Buy, 51_00, 15, "buyer".to_string());
        let res = order_book.register_offer(matching_bid);
        assert_eq!(res.len(), 2);

        println!("{res:?}");
        assert_eq!(res[1].buyer, "buyer".to_string());
        assert_eq!(res[1].seller, "seller_1".to_string());
        assert_eq!(res[1].volume, 10);
        assert_eq!(res[1].price, 50_00);

        assert_eq!(res[0].buyer, "buyer".to_string());
        assert_eq!(res[0].seller, "seller_2".to_string());
        assert_eq!(res[0].volume, 5);
        assert_eq!(res[0].price, 51_00);
    }
}

#[cfg(test)]
mod test_bid_and_offer {
    use std::{cmp::Ordering, time::Instant};

    use uuid::Uuid;

    use crate::{market::Direction, order_book::Offer};

    use super::{Bid, Order};

    #[test]
    fn test_bids_ordering() {
        fn build_bid(price: usize) -> Bid {
            Bid(Order {
                direction: Direction::Buy,
                price,
                owner: Uuid::new_v4().to_string(),
                volume: 10,
                timestamp: Instant::now(),
            })
        }

        let bid = build_bid(50_00);
        assert_eq!(bid.cmp(&bid), Ordering::Equal);

        let more_expensive_bid = build_bid(50_01);
        assert_eq!(bid.cmp(&more_expensive_bid), Ordering::Greater);

        let less_expensive_bid = build_bid(49_99);
        assert_eq!(bid.cmp(&less_expensive_bid), Ordering::Less);

        let same_price_but_older_bid = build_bid(50_00);
        assert_eq!(bid.cmp(&same_price_but_older_bid), Ordering::Greater);
    }

    #[test]
    fn test_offers_ordering() {
        fn build_offer(price: usize) -> Offer {
            Offer(Order {
                direction: Direction::Sell,
                price,
                owner: Uuid::new_v4().to_string(),
                volume: 10,
                timestamp: Instant::now(),
            })
        }

        let offer = build_offer(50_00);
        assert_eq!(offer.cmp(&offer), Ordering::Equal);

        let more_expensive_offer = build_offer(50_01);
        assert_eq!(offer.cmp(&more_expensive_offer), Ordering::Less);

        let less_expensive_offer = build_offer(49_99);
        assert_eq!(offer.cmp(&less_expensive_offer), Ordering::Greater);

        let same_price_but_older_offer = build_offer(50_00);
        assert_eq!(offer.cmp(&same_price_but_older_offer), Ordering::Greater);
    }
}

#[cfg(test)]
mod test_trade_leg {
    use chrono::Utc;

    use crate::{market::Direction, order_book::TradeLeg};

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
