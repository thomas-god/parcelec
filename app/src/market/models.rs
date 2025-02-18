use std::future::Future;

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use crate::{game::delivery_period::DeliveryPeriodId, player::PlayerId};

use super::{
    order_book::{OrderRequest, Trade, TradeLeg},
    MarketMessage, OBS,
};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Direction {
    Buy,
    Sell,
}

pub trait MarketService: Clone + Send + Sync + 'static {
    fn open_market(&self, delivery_period: DeliveryPeriodId) -> impl Future<Output = ()> + Send;

    fn close_market(
        &self,
        delivery_period: DeliveryPeriodId,
    ) -> impl Future<Output = Vec<Trade>> + Send;

    fn register_player(
        &self,
        player: PlayerId,
    ) -> impl Future<Output = (Vec<TradeLeg>, OBS)> + Send;

    fn new_order(&self, request: OrderRequest) -> impl Future<Output = ()> + Send;

    fn delete_order(&self, order_id: String) -> impl Future<Output = ()> + Send;
}

#[derive(Debug, Clone)]
pub struct Service {
    tx: mpsc::Sender<MarketMessage>,
}

impl Service {
    pub fn new(tx: mpsc::Sender<MarketMessage>) -> Service {
        Service { tx }
    }
}

impl MarketService for Service {
    async fn open_market(&self, delivery_period: DeliveryPeriodId) {
        let _ = self
            .tx
            .send(MarketMessage::OpenMarket(delivery_period))
            .await;
    }

    async fn close_market(&self, delivery_period: DeliveryPeriodId) -> Vec<Trade> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(MarketMessage::CloseMarket {
                period_id: delivery_period,
                tx_back: tx,
            })
            .await;

        rx.await.unwrap_or(Vec::new())
    }

    async fn register_player(&self, player: PlayerId) -> (Vec<TradeLeg>, OBS) {
        let (tx, rx) = oneshot::channel();

        let _ = self
            .tx
            .send(MarketMessage::NewPlayerConnection {
                player_id: player,
                tx_back: tx,
            })
            .await;

        rx.await.unwrap_or((
            Vec::new(),
            OBS {
                bids: Vec::new(),
                offers: Vec::new(),
            },
        ))
    }

    async fn new_order(&self, request: OrderRequest) {
        let _ = self.tx.send(MarketMessage::OrderRequest(request)).await;
    }

    async fn delete_order(&self, order_id: String) {
        let _ = self
            .tx
            .send(MarketMessage::OrderDeletionRequest { order_id })
            .await;
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use crate::{
        game::delivery_period::DeliveryPeriodId,
        market::{
            models::{Direction, MarketService, OBS},
            order_book::OrderRequest,
            MarketMessage,
        },
        player::PlayerId,
    };

    use super::Service;

    #[tokio::test]
    async fn test_open_market() {
        let (tx, mut rx) = mpsc::channel(16);
        let service = Service::new(tx);

        let _ = service.open_market(DeliveryPeriodId::from(0)).await;

        let Some(MarketMessage::OpenMarket(delivery_period)) = rx.recv().await else {
            unreachable!();
        };
        assert_eq!(delivery_period, DeliveryPeriodId::from(0));
    }

    #[tokio::test]
    async fn test_close_market_ok() {
        let (tx, mut rx) = mpsc::channel(128);
        let service = Service::new(tx);

        tokio::spawn(async move {
            let Some(MarketMessage::CloseMarket {
                period_id: _,
                tx_back,
            }) = rx.recv().await
            else {
                unreachable!()
            };
            let _ = tx_back.send(Vec::new());
        });

        let res = service.close_market(DeliveryPeriodId::from(0)).await;
        assert_eq!(res.len(), 0);
    }
    #[tokio::test]
    async fn test_close_market_err() {
        let (tx, mut rx) = mpsc::channel(128);
        let service = Service::new(tx);

        // Close receiving end to simulate err
        rx.close();

        let res = service.close_market(DeliveryPeriodId::from(0)).await;
        // Should still receive an empty vec
        assert_eq!(res.len(), 0);
    }
    #[tokio::test]
    async fn test_register_player_ok() {
        let (tx, mut rx) = mpsc::channel(128);
        let service = Service::new(tx);

        tokio::spawn(async move {
            let Some(MarketMessage::NewPlayerConnection {
                player_id: _,
                tx_back,
            }) = rx.recv().await
            else {
                unreachable!()
            };
            let _ = tx_back.send((
                Vec::new(),
                OBS {
                    bids: Vec::new(),
                    offers: Vec::new(),
                },
            ));
        });

        let res = service.close_market(DeliveryPeriodId::from(0)).await;
        assert_eq!(res.len(), 0);
    }
    #[tokio::test]
    async fn test_register_player_err() {
        let (tx, mut rx) = mpsc::channel(128);
        let service = Service::new(tx);

        // Close receiving end to simulate err
        rx.close();

        let (trades, obs) = service.register_player(PlayerId::default()).await;
        // Should still receive an empty vec
        assert_eq!(trades.len(), 0);
        assert_eq!(obs.offers.len(), 0);
        assert_eq!(obs.bids.len(), 0);
    }

    #[tokio::test]
    async fn test_new_order() {
        let (tx, mut rx) = mpsc::channel(16);
        let service = Service::new(tx);
        let request = OrderRequest {
            direction: Direction::Buy,
            owner: PlayerId::default(),
            volume: 100,
            price: 10_00,
        };

        let _ = service.new_order(request.clone()).await;

        let Some(MarketMessage::OrderRequest(req)) = rx.recv().await else {
            unreachable!();
        };
        assert_eq!(req, request);
    }

    #[tokio::test]
    async fn test_delete_order() {
        let (tx, mut rx) = mpsc::channel(16);
        let service = Service::new(tx);
        let order_id = String::from("toto");

        let _ = service.delete_order(order_id).await;

        let Some(MarketMessage::OrderDeletionRequest { order_id }) = rx.recv().await else {
            unreachable!();
        };
        assert_eq!(order_id, String::from("toto"));
    }
}
