#[allow(unused_imports)] // To be used in mock!
use std::future::Future;

use tokio::sync::{mpsc, oneshot};

use crate::{
    game::delivery_period::DeliveryPeriodId,
    market::{
        Market, OBS,
        order_book::{OrderRequest, Trade, TradeLeg},
    },
    player::PlayerId,
};

use super::MarketMessage;

#[derive(Debug, Clone)]
pub struct MarketService {
    tx: mpsc::Sender<MarketMessage>,
}

impl MarketService {
    pub fn new(tx: mpsc::Sender<MarketMessage>) -> MarketService {
        MarketService { tx }
    }
}

impl Market for MarketService {
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

    async fn get_market_snapshot(&self, player: PlayerId) -> (Vec<TradeLeg>, OBS) {
        let (tx, rx) = oneshot::channel();

        let _ = self
            .tx
            .send(MarketMessage::GetMarketSnapshot {
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
mockall::mock! {
    pub MarketService {}

    impl Market for MarketService {
        fn open_market(&self, delivery_period: DeliveryPeriodId) -> impl Future<Output = ()> + Send;

        fn close_market(
            &self,
            delivery_period: DeliveryPeriodId,
        ) -> impl Future<Output = Vec<Trade>> + Send;

        fn get_market_snapshot(
            &self,
            player: PlayerId,
        ) -> impl Future<Output = (Vec<TradeLeg>, OBS)> + Send;

        fn new_order(&self, request: OrderRequest) -> impl Future<Output = ()> + Send;

        fn delete_order(&self, order_id: String) -> impl Future<Output = ()> + Send;

    }

    impl Clone for MarketService {
        fn clone(&self) -> Self;
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use crate::{
        game::delivery_period::DeliveryPeriodId,
        market::{
            Direction, MarketMessage,
            infra::service::{Market, OBS},
            order_book::OrderRequest,
        },
        player::PlayerId,
        utils::units::{Energy, EnergyCost},
    };

    use super::MarketService;

    #[tokio::test]
    async fn test_open_market() {
        let (tx, mut rx) = mpsc::channel(16);
        let service = MarketService::new(tx);

        let _ = service.open_market(DeliveryPeriodId::from(0)).await;

        let Some(MarketMessage::OpenMarket(delivery_period)) = rx.recv().await else {
            unreachable!();
        };
        assert_eq!(delivery_period, DeliveryPeriodId::from(0));
    }

    #[tokio::test]
    async fn test_close_market_ok() {
        let (tx, mut rx) = mpsc::channel(128);
        let service = MarketService::new(tx);

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
        let service = MarketService::new(tx);

        // Close receiving end to simulate err
        rx.close();

        let res = service.close_market(DeliveryPeriodId::from(0)).await;
        // Should still receive an empty vec
        assert_eq!(res.len(), 0);
    }
    #[tokio::test]
    async fn test_register_player_ok() {
        let (tx, mut rx) = mpsc::channel(128);
        let service = MarketService::new(tx);

        tokio::spawn(async move {
            let Some(MarketMessage::GetMarketSnapshot {
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
        let service = MarketService::new(tx);

        // Close receiving end to simulate err
        rx.close();

        let (trades, obs) = service.get_market_snapshot(PlayerId::default()).await;
        // Should still receive an empty vec
        assert_eq!(trades.len(), 0);
        assert_eq!(obs.offers.len(), 0);
        assert_eq!(obs.bids.len(), 0);
    }

    #[tokio::test]
    async fn test_new_order() {
        let (tx, mut rx) = mpsc::channel(16);
        let service = MarketService::new(tx);
        let request = OrderRequest {
            direction: Direction::Buy,
            owner: PlayerId::default(),
            volume: Energy::from(100),
            price: EnergyCost::from(10),
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
        let service = MarketService::new(tx);
        let order_id = String::from("toto");

        let _ = service.delete_order(order_id).await;

        let Some(MarketMessage::OrderDeletionRequest { order_id }) = rx.recv().await else {
            unreachable!();
        };
        assert_eq!(order_id, String::from("toto"));
    }
}
