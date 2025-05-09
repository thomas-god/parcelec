use std::{collections::HashMap, fmt::Debug};

use futures_util::future::join_all;
use tokio::sync::{mpsc, oneshot, watch};
use tokio_util::sync::CancellationToken;

use crate::{
    game::{GameId, delivery_period::DeliveryPeriodId},
    market::{
        MarketContext, MarketState, OBS, OrderRepr,
        order_book::{OrderBook, OrderRequest, Trade, TradeLeg},
    },
    player::{PlayerConnections, PlayerId, PlayerMessage},
};

use super::MarketService;

#[derive(Debug)]
pub enum MarketMessage {
    GetMarketSnapshot {
        player_id: PlayerId,
        tx_back: oneshot::Sender<(Vec<TradeLeg>, OBS)>,
    },
    OpenMarket(DeliveryPeriodId),
    CloseMarket {
        period_id: DeliveryPeriodId,
        tx_back: oneshot::Sender<Vec<Trade>>,
    },
    OrderRequest(OrderRequest),
    OrderDeletionRequest {
        order_id: String,
    },
}

pub struct MarketActor<PC: PlayerConnections> {
    game_id: GameId,
    delivery_period: DeliveryPeriodId,
    state: MarketState,
    state_sender: watch::Sender<MarketState>,
    rx: mpsc::Receiver<MarketMessage>,
    tx: mpsc::Sender<MarketMessage>,
    order_book: OrderBook,
    players: Vec<PlayerId>,
    players_connections: PC,
    past_trades: HashMap<DeliveryPeriodId, Vec<Trade>>,
    cancellation_token: CancellationToken,
}

impl<PC: PlayerConnections> MarketActor<PC> {
    fn new(
        game_id: GameId,
        state: MarketState,
        delivery_period: DeliveryPeriodId,
        players_connections: PC,
        cancellation_token: CancellationToken,
    ) -> MarketActor<PC> {
        let (state_tx, _) = watch::channel(state);
        let (tx, rx) = mpsc::channel::<MarketMessage>(128);

        MarketActor {
            game_id,
            state,
            delivery_period,
            state_sender: state_tx,
            rx,
            tx,
            players: Vec::new(),
            players_connections,
            order_book: OrderBook::new(),
            past_trades: HashMap::new(),
            cancellation_token,
        }
    }

    pub fn start(
        game_id: &GameId,
        players_connections: PC,
        cancellation_token: CancellationToken,
    ) -> MarketContext<MarketService> {
        let mut market = MarketActor::new(
            game_id.clone(),
            MarketState::Closed,
            DeliveryPeriodId::default(),
            players_connections,
            cancellation_token,
        );
        let context = market.get_context();

        tokio::spawn(async move {
            market.process().await;
        });
        context
    }

    pub fn get_context(&self) -> MarketContext<MarketService> {
        MarketContext {
            service: MarketService::new(self.tx.clone()),
            state_rx: self.state_sender.subscribe(),
        }
    }

    pub async fn process(&mut self) {
        loop {
            tokio::select! {
                Some(message) = self.rx.recv() => {
                    self.process_message(message).await;
                }
                _ = self.cancellation_token.cancelled() => {
                    tracing::info!("Market actor for game {:?} terminated", self.game_id);
                    break;
                }
            }
        }
    }

    #[tracing::instrument(name = "MarketActor::process_message", skip(self))]
    async fn process_message(&mut self, message: MarketMessage) {
        match (&self.state, message) {
            (_, MarketMessage::GetMarketSnapshot { player_id, tx_back }) => {
                self.players.push(player_id.clone());
                let _ = tx_back.send((self.player_trades(&player_id), self.player_obs(&player_id)));
            }
            (MarketState::Open, MarketMessage::OrderRequest(request)) => {
                self.process_order_request(request).await
            }
            (MarketState::Open, MarketMessage::OrderDeletionRequest { order_id }) => {
                tracing::info!("Order deletion request for order: {order_id:?}");
                self.order_book.remove_offer(order_id);
                self.send_order_book_snapshot_to_all().await;
            }

            (MarketState::Closed, MarketMessage::OpenMarket(period_id)) => {
                if period_id == self.delivery_period {
                    self.state = MarketState::Open;
                    self.delivery_period = self.delivery_period.next();
                    let _ = self.state_sender.send(MarketState::Open);
                }
            }
            (MarketState::Open, MarketMessage::CloseMarket { tx_back, period_id }) => {
                if period_id == self.delivery_period {
                    self.close_market(period_id, tx_back).await;
                }
            }
            (MarketState::Closed, MarketMessage::CloseMarket { period_id, tx_back }) => {
                if let Some(trades) = self.past_trades.get(&period_id) {
                    let _ = tx_back.send(trades.clone());
                }
            }
            (MarketState::Closed, MarketMessage::OrderRequest(_))
            | (MarketState::Closed, MarketMessage::OrderDeletionRequest { order_id: _ }) => {
                tracing::warn!(
                    "Market closed, cannot process new order request, or deletion request"
                );
            }
            (MarketState::Open, MarketMessage::OpenMarket(_)) => {
                tracing::warn!("Market is already open");
            }
        }
    }

    async fn close_market(
        &mut self,
        period_id: DeliveryPeriodId,
        tx_back: oneshot::Sender<Vec<Trade>>,
    ) {
        tracing::info!(
            game_id = ?self.game_id,
            period_id = ?period_id,
            "Closing market for period"
        );

        // Drain trades from order book and store them
        let trades = self.order_book.drain();
        self.past_trades.insert(period_id, trades.clone());

        // Update market state
        self.state = MarketState::Closed;
        if let Err(err) = self.state_sender.send(MarketState::Closed) {
            tracing::error!(
                game_id = ?self.game_id,
                period_id = ?period_id,
                "Failed to broadcast market state change: {:?}", err
            );
        }

        // Send trades back to requester
        if let Err(err) = tx_back.send(trades) {
            tracing::error!(
                game_id = ?self.game_id,
                period_id = ?period_id,
                trade_count = self.order_book.trades.len(),
                "Failed to send trades back to requester: {:?}", err
            );
        }

        // Notify all players about the updated state
        self.send_order_book_snapshot_to_all().await;
        self.send_empty_trade_list_to_all().await;
    }

    async fn send_order_book_snapshot_to_all(&self) {
        join_all(
            self.players
                .iter()
                .map(|player_id| self.send_order_book_snapshot_to_player(player_id))
                .collect::<Vec<_>>(),
        )
        .await;
    }

    async fn send_empty_trade_list_to_all(&self) {
        self.players_connections
            .send_to_all_players(
                &self.game_id,
                PlayerMessage::TradeList { trades: Vec::new() },
            )
            .await;
    }

    fn player_obs(&self, player_id: &PlayerId) -> OBS {
        let snapshot = self.order_book.snapshot();
        OBS {
            bids: snapshot
                .bids
                .iter()
                .map(|bid| OrderRepr::from_bid(bid, Some(player_id)))
                .collect(),
            offers: snapshot
                .offers
                .iter()
                .map(|offer| OrderRepr::from_offer(offer, Some(player_id)))
                .collect(),
        }
    }

    async fn send_order_book_snapshot_to_player(&self, player_id: &PlayerId) {
        let obs = self.player_obs(player_id);
        let message = PlayerMessage::OrderBookSnapshot {
            bids: obs.bids,
            offers: obs.offers,
        };
        let _ = self
            .players_connections
            .send_to_player(&self.game_id, player_id, message)
            .await;
    }

    fn player_trades(&self, player_id: &PlayerId) -> Vec<TradeLeg> {
        self.order_book
            .trades
            .iter()
            .flat_map(|trade| trade.for_player(player_id))
            .collect()
    }

    async fn notify_trades(&self, trades: &[Trade]) {
        if trades.is_empty() {
            return;
        }
        for trade_leg in trades.iter().flat_map(|trade| trade.split()) {
            let conns = self.players_connections.clone();
            let game = self.game_id.clone();
            let owner = trade_leg.owner.clone();
            tokio::spawn(async move {
                conns
                    .send_to_player(&game, &owner, PlayerMessage::NewTrade(trade_leg))
                    .await;
            });
        }
    }

    #[tracing::instrument(name = "ActorMarket::process_order_request", skip(self))]
    async fn process_order_request(&mut self, request: OrderRequest) {
        let trades = self.order_book.register_order_request(request);
        tracing::info!("New trades: {trades:?}");

        // Update all players with new order book state
        self.send_order_book_snapshot_to_all().await;

        // Notify players about their trades
        self.notify_trades(&trades).await;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::{
        mpsc::{self},
        oneshot, watch,
    };
    use tokio_util::sync::CancellationToken;

    use crate::{
        game::{GameId, delivery_period::DeliveryPeriodId},
        market::{Direction, MarketState, order_book::OrderRequest},
        player::{PlayerConnections, PlayerId},
        utils::units::{Energy, EnergyCost},
    };

    use super::{MarketActor, MarketMessage, PlayerMessage};

    #[derive(Debug, Clone)]
    pub struct MockPlayerConnections {
        pub tx_send_to_player: mpsc::Sender<(PlayerId, PlayerMessage)>,
        pub tx_send_to_all_players: mpsc::Sender<PlayerMessage>,
    }
    impl MockPlayerConnections {
        pub fn new() -> (
            MockPlayerConnections,
            mpsc::Receiver<(PlayerId, PlayerMessage)>,
            mpsc::Receiver<PlayerMessage>,
        ) {
            let (tx_send_to_player, rx_send_to_player) = mpsc::channel(16);
            let (tx_send_to_all_players, rx_send_to_all_players) = mpsc::channel(16);
            (
                MockPlayerConnections {
                    tx_send_to_player,
                    tx_send_to_all_players,
                },
                rx_send_to_player,
                rx_send_to_all_players,
            )
        }
    }
    impl PlayerConnections for MockPlayerConnections {
        async fn send_to_all_players(&self, _game: &GameId, message: PlayerMessage) {
            let _ = self.tx_send_to_all_players.send(message).await;
        }
        async fn send_to_player(&self, _game: &GameId, player: &PlayerId, message: PlayerMessage) {
            let _ = self.tx_send_to_player.send((player.clone(), message)).await;
        }
    }

    fn start_market_actor(
        game_id: &GameId,
        connections: MockPlayerConnections,
    ) -> (mpsc::Sender<MarketMessage>, watch::Receiver<MarketState>) {
        let token = CancellationToken::new();
        let mut market = MarketActor::new(
            game_id.clone(),
            MarketState::Open,
            DeliveryPeriodId::from(0),
            connections,
            token,
        );
        let tx = market.tx.clone();
        let watch = market.state_sender.subscribe();
        tokio::spawn(async move {
            market.process().await;
        });
        (tx, watch)
    }

    async fn register_player(market: mpsc::Sender<MarketMessage>) -> PlayerId {
        let player_id = PlayerId::default();
        let (tx, _) = oneshot::channel();
        let _ = market
            .send(MarketMessage::GetMarketSnapshot {
                player_id: player_id.clone(),
                tx_back: tx,
            })
            .await;
        player_id
    }

    #[tokio::test]
    async fn test_process_request_order() {
        let game_id = GameId::default();
        let (conn_tx, mut conn_rx, ..) = MockPlayerConnections::new();
        let (tx, _) = start_market_actor(&game_id, conn_tx.clone());
        let player_id = register_player(tx.clone()).await;

        // Send order request to market actor
        tx.send(MarketMessage::OrderRequest(OrderRequest {
            direction: Direction::Buy,
            price: EnergyCost::from(50),
            volume: Energy::from(10),
            owner: player_id.clone(),
        }))
        .await
        .unwrap();

        // The list of offers has been updated to contains our new offer (that we own)
        let Some((target_player_id, PlayerMessage::OrderBookSnapshot { bids, offers })) =
            conn_rx.recv().await
        else {
            unreachable!("Expected PlayerMessage::PublicOffers")
        };
        assert_eq!(target_player_id, player_id);
        assert_eq!(bids.len(), 1);
        assert_eq!(offers.len(), 0);
        assert!(bids.first().unwrap().owned);
    }

    #[tokio::test]
    async fn test_process_delete_order() {
        let game_id = GameId::default();
        let (conn_tx, mut conn_rx, ..) = MockPlayerConnections::new();
        let (tx, _) = start_market_actor(&game_id, conn_tx.clone());
        let player_id = register_player(tx.clone()).await;

        // Send order request to market actor
        tx.send(MarketMessage::OrderRequest(OrderRequest {
            direction: Direction::Buy,
            price: EnergyCost::from(50),
            volume: Energy::from(10),
            owner: player_id.clone(),
        }))
        .await
        .unwrap();

        // The list of offers has been updated to contains our new offer
        let Some((_, PlayerMessage::OrderBookSnapshot { bids, offers })) = conn_rx.recv().await
        else {
            unreachable!("Expected PlayerMessage::PublicOffers")
        };
        assert_eq!(bids.len(), 1);
        assert_eq!(offers.len(), 0);
        let order_id = bids.first().unwrap().order_id.clone();

        // Send request to delete order
        tx.send(MarketMessage::OrderDeletionRequest { order_id })
            .await
            .unwrap();
        // The list of offers should be empty
        let Some((_, PlayerMessage::OrderBookSnapshot { bids, offers })) = conn_rx.recv().await
        else {
            unreachable!("Expected PlayerMessage::PublicOffers")
        };
        assert_eq!(bids.len(), 0);
        assert_eq!(offers.len(), 0);
    }

    #[tokio::test]
    async fn test_match_offers() {
        let game_id = GameId::default();
        let (conn_tx, mut conn_rx, ..) = MockPlayerConnections::new();
        let (tx, _) = start_market_actor(&game_id, conn_tx.clone());
        let buyer_id = register_player(tx.clone()).await;
        let seller_id = register_player(tx.clone()).await;

        // Send BUY order
        tx.send(MarketMessage::OrderRequest(OrderRequest {
            direction: Direction::Buy,
            volume: Energy::from(10),
            price: EnergyCost::from(50),
            owner: buyer_id.clone(),
        }))
        .await
        .unwrap();
        // Flush the corresponding OBS updates (1 per player)
        conn_rx.recv().await.unwrap();
        conn_rx.recv().await.unwrap();

        // Send SELL order
        tx.send(MarketMessage::OrderRequest(OrderRequest {
            direction: Direction::Sell,
            volume: Energy::from(10),
            price: EnergyCost::from(50),
            owner: seller_id.clone(),
        }))
        .await
        .unwrap();

        // The order book snapshots should be empty
        for _ in 0..2 {
            let Some((_, PlayerMessage::OrderBookSnapshot { bids, offers })) = conn_rx.recv().await
            else {
                unreachable!("Expected PlayerMessage::PublicOffers")
            };
            assert_eq!(bids.len(), 0);
            assert_eq!(offers.len(), 0);
        }

        // Each player should receive its own trade leg
        for _ in 0..2 {
            let Some((player_id, PlayerMessage::NewTrade(trade))) = conn_rx.recv().await else {
                unreachable!("Expected PlayerMessage::NewTrade")
            };
            if player_id == buyer_id {
                assert_eq!(trade.direction, Direction::Buy)
            } else if player_id == seller_id {
                assert_eq!(trade.direction, Direction::Sell)
            } else {
                unreachable!()
            }
        }
    }

    #[tokio::test]
    async fn test_closed_market_does_not_process_order_request() {
        let game_id = GameId::default();
        let (conn_tx, mut conn_rx, ..) = MockPlayerConnections::new();
        let token = CancellationToken::new();
        let mut market = MarketActor::new(
            game_id,
            MarketState::Closed,
            DeliveryPeriodId::from(0),
            conn_tx,
            token,
        );
        let tx = market.tx.clone();
        tokio::spawn(async move {
            market.process().await;
        });
        let player_id = register_player(tx.clone()).await;

        // Send an OrderRequest to the market
        let _ = tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Buy,
                volume: Energy::from(10),
                price: EnergyCost::from(50),
                owner: player_id.to_owned(),
            }))
            .await;

        // We should not receive an order book snapshot

        tokio::select! {
        _ = conn_rx.recv() => {
            unreachable!("Should not have received a message");
        }
        _ = tokio::time::sleep(Duration::from_micros(1)) => {}
        };
    }

    #[tokio::test]
    async fn test_close_market_and_reopen() {
        let game_id = GameId::default();
        let (conn_tx, mut conn_rx, ..) = MockPlayerConnections::new();
        let (tx, _) = start_market_actor(&game_id, conn_tx.clone());
        let player_id = register_player(tx.clone()).await;

        // Close the market
        let (tx_back, _) = oneshot::channel();
        let _ = tx
            .send(MarketMessage::CloseMarket {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;

        // Reopen the market
        let _ = tx
            .send(MarketMessage::OpenMarket(DeliveryPeriodId::from(0)))
            .await;

        // Send an OrderRequest to the market
        let _ = tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Buy,
                volume: Energy::from(10),
                price: EnergyCost::from(50),
                owner: player_id.to_owned(),
            }))
            .await;

        // We should receive an obs
        let Some(_) = conn_rx.recv().await else {
            unreachable!("Expected PlayerMessage::PublicOffers")
        };
    }

    #[tokio::test]
    async fn test_close_market_receive_trades() {
        let game_id = GameId::default();
        let (conn_tx, ..) = MockPlayerConnections::new();
        let (tx, _) = start_market_actor(&game_id, conn_tx.clone());
        let player_id = register_player(tx.clone()).await;

        // Make a trade with ourself
        let _ = tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Buy,
                volume: Energy::from(10),
                price: EnergyCost::from(50),
                owner: player_id.to_owned(),
            }))
            .await;
        let _ = tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Sell,
                volume: Energy::from(10),
                price: EnergyCost::from(50),
                owner: player_id.to_owned(),
            }))
            .await;

        // Close the market and receive the trade list back
        let (tx_back, rx_back) = oneshot::channel();
        let _ = tx
            .send(MarketMessage::CloseMarket {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;
        let trades = rx_back
            .await
            .expect("Should have received a list of trades");
        assert_eq!(trades.len(), 1);
    }

    #[tokio::test]
    async fn test_close_market_send_empty_obs_and_trade_list() {
        let game_id = GameId::default();
        let (conn_tx, mut rx_player, mut rx_all) = MockPlayerConnections::new();
        let (tx, _) = start_market_actor(&game_id, conn_tx.clone());
        let player_id = register_player(tx.clone()).await;

        // Send an order
        let _ = tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Buy,
                volume: Energy::from(10),
                price: EnergyCost::from(50),
                owner: player_id.to_owned(),
            }))
            .await;
        let _ = rx_player.recv().await;

        // Close the market
        let (tx_back, rx_back) = oneshot::channel();
        let _ = tx
            .send(MarketMessage::CloseMarket {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;

        // Flush the trade list
        let _ = rx_back.await;

        // OBS should be empty
        let Some((_, PlayerMessage::OrderBookSnapshot { bids, offers })) = rx_player.recv().await
        else {
            unreachable!()
        };
        assert_eq!(bids.len(), 0);
        assert_eq!(offers.len(), 0);

        // Trade list should be empty
        let Some(PlayerMessage::TradeList { trades }) = rx_all.recv().await else {
            unreachable!()
        };
        assert_eq!(trades.len(), 0);
    }

    #[tokio::test]
    async fn test_market_state_watch() {
        let game_id = GameId::default();
        let (conn_tx, ..) = MockPlayerConnections::new();
        let (tx, mut state_rx) = start_market_actor(&game_id, conn_tx.clone());
        register_player(tx.clone()).await;

        assert_eq!(*state_rx.borrow(), MarketState::Open);

        // Close the market
        let (tx_back, _) = oneshot::channel();
        let _ = tx
            .send(MarketMessage::CloseMarket {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;
        assert!(state_rx.changed().await.is_ok());
        assert_eq!(*state_rx.borrow_and_update(), MarketState::Closed);

        // Reopen the market
        let _ = tx
            .send(MarketMessage::OpenMarket(DeliveryPeriodId::from(0)))
            .await;
        assert!(state_rx.changed().await.is_ok());
        assert_eq!(*state_rx.borrow_and_update(), MarketState::Open);
    }

    #[tokio::test]
    async fn test_try_closing_market_wrong_period_id_does_not_close_it() {
        let game_id = GameId::default();
        let (conn_tx, ..) = MockPlayerConnections::new();
        let token = CancellationToken::new();
        let mut market = MarketActor::new(
            game_id,
            MarketState::Open,
            DeliveryPeriodId::from(1),
            conn_tx,
            token,
        );
        let market_tx = market.tx.clone();
        let mut state_rx = market.state_sender.subscribe();
        tokio::spawn(async move {
            market.process().await;
        });

        assert_eq!(*state_rx.borrow_and_update(), MarketState::Open);

        // Try closing the market with period ID greater than the actual one
        let (tx_back, _) = oneshot::channel();
        let _ = market_tx
            .send(MarketMessage::CloseMarket {
                tx_back,
                period_id: DeliveryPeriodId::from(2),
            })
            .await;
        tokio::time::sleep(Duration::from_micros(1)).await;
        assert_eq!(*state_rx.borrow_and_update(), MarketState::Open);

        // Try closing the market with period ID smaller than the actual one
        let (tx_back, _) = oneshot::channel();
        let _ = market_tx
            .send(MarketMessage::CloseMarket {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;
        tokio::time::sleep(Duration::from_micros(1)).await;
        assert_eq!(*state_rx.borrow_and_update(), MarketState::Open);
    }

    #[tokio::test]
    async fn test_opening_market_wrong_period_id_does_not_open_it() {
        let game_id = GameId::default();
        let (conn_tx, ..) = MockPlayerConnections::new();
        let token = CancellationToken::new();
        let mut market = MarketActor::new(
            game_id,
            MarketState::Closed,
            DeliveryPeriodId::from(1),
            conn_tx,
            token,
        );
        let market_tx = market.tx.clone();
        let mut state_rx = market.state_sender.subscribe();
        tokio::spawn(async move {
            market.process().await;
        });

        assert_eq!(*state_rx.borrow_and_update(), MarketState::Closed);

        // Try openning the market with period ID greater than the actual one
        let _ = market_tx
            .send(MarketMessage::OpenMarket(DeliveryPeriodId::from(2)))
            .await;
        tokio::time::sleep(Duration::from_micros(1)).await;
        assert_eq!(*state_rx.borrow_and_update(), MarketState::Closed);

        // Try closing the market with period ID smaller than the actual one
        let _ = market_tx
            .send(MarketMessage::OpenMarket(DeliveryPeriodId::from(0)))
            .await;
        tokio::time::sleep(Duration::from_micros(1)).await;
        assert_eq!(*state_rx.borrow_and_update(), MarketState::Closed);
    }

    #[tokio::test]
    async fn test_open_market_then_close_next_period() {
        let game_id = GameId::default();
        let (conn_tx, ..) = MockPlayerConnections::new();
        let token = CancellationToken::new();
        let mut market = MarketActor::new(
            game_id,
            MarketState::Closed,
            DeliveryPeriodId::from(1),
            conn_tx,
            token,
        );
        let market_tx = market.tx.clone();
        let mut state_rx = market.state_sender.subscribe();
        tokio::spawn(async move {
            market.process().await;
        });

        // Open the market
        let _ = market_tx
            .send(MarketMessage::OpenMarket(DeliveryPeriodId::from(1)))
            .await;
        assert!(state_rx.changed().await.is_ok());
        assert_eq!(*state_rx.borrow_and_update(), MarketState::Open);

        // Close the market with next period id
        let (tx_back, _) = oneshot::channel();
        let _ = market_tx
            .send(MarketMessage::CloseMarket {
                tx_back,
                period_id: DeliveryPeriodId::from(2),
            })
            .await;
        assert!(state_rx.changed().await.is_ok());
        assert_eq!(*state_rx.borrow_and_update(), MarketState::Closed);
    }

    #[tokio::test]
    async fn test_closing_twice_should_return_the_same_trades() {
        let game_id = GameId::default();
        let (conn_tx, ..) = MockPlayerConnections::new();
        let (tx, mut state_rx) = start_market_actor(&game_id, conn_tx);
        let player_id = register_player(tx.clone()).await;

        // Make a trade with ourself
        let _ = tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Buy,
                volume: Energy::from(10),
                price: EnergyCost::from(50),
                owner: player_id.to_owned(),
            }))
            .await;
        let _ = tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Sell,
                volume: Energy::from(10),
                price: EnergyCost::from(50),
                owner: player_id.to_owned(),
            }))
            .await;

        // Close the market and receive the trade list back
        let (tx_back, rx_back) = oneshot::channel();
        let _ = tx
            .send(MarketMessage::CloseMarket {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;
        let trades = rx_back
            .await
            .expect("Should have received a list of trades");
        assert_eq!(trades.len(), 1);
        assert!(state_rx.changed().await.is_ok());
        assert_eq!(*state_rx.borrow_and_update(), MarketState::Closed);

        // Close the market again and receive the same trades
        let (tx_back, rx_back) = oneshot::channel();
        let _ = tx
            .send(MarketMessage::CloseMarket {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;
        let same_trades = rx_back
            .await
            .expect("Should have received a list of trades");
        assert_eq!(trades, same_trades);
    }

    #[tokio::test]
    async fn test_market_snapshot() {
        let game_id = GameId::default();
        let (conn_tx, ..) = MockPlayerConnections::new();
        let (tx, _) = start_market_actor(&game_id, conn_tx.clone());
        let player_id = register_player(tx.clone()).await;

        // Request market snapshot
        let (tx_back, rx_back) = oneshot::channel();
        let _ = tx
            .send(MarketMessage::GetMarketSnapshot {
                player_id: player_id.clone(),
                tx_back,
            })
            .await;

        // Verify the snapshot
        let (trades, obs) = rx_back.await.expect("Should have received a snapshot");
        assert!(trades.is_empty());
        assert!(obs.bids.is_empty());
        assert!(obs.offers.is_empty());

        // Send an order
        let _ = tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Buy,
                volume: Energy::from(10),
                price: EnergyCost::from(50),
                owner: player_id.clone(),
            }))
            .await;

        // Request market snapshot again
        let (tx_back, rx_back) = oneshot::channel();
        let _ = tx
            .send(MarketMessage::GetMarketSnapshot {
                player_id: player_id.clone(),
                tx_back,
            })
            .await;

        // Verify the snapshot
        let (trades, obs) = rx_back.await.expect("Should have received a snapshot");
        assert!(trades.is_empty());
        assert_eq!(obs.bids.len(), 1);
        assert!(obs.offers.is_empty());

        // Send a matching order to have a trade
        let _ = tx
            .send(MarketMessage::OrderRequest(OrderRequest {
                direction: Direction::Sell,
                volume: Energy::from(10),
                price: EnergyCost::from(50),
                owner: player_id.clone(),
            }))
            .await;

        // Verify the snapshot
        let (tx_back, rx_back) = oneshot::channel();
        let _ = tx
            .send(MarketMessage::GetMarketSnapshot {
                player_id: player_id.clone(),
                tx_back,
            })
            .await;
        let (trades, obs) = rx_back.await.expect("Should have received a snapshot");
        assert_eq!(trades.len(), 2); // 1 trade = 2 trade legs
        assert!(obs.bids.is_empty());
        assert!(obs.offers.is_empty());
    }

    #[tokio::test]
    async fn test_terminate_actor() {
        let (connections, ..) = MockPlayerConnections::new();
        let token = CancellationToken::new();
        let mut market = MarketActor::new(
            GameId::default(),
            MarketState::Open,
            DeliveryPeriodId::from(0),
            connections,
            token.clone(),
        );
        let handle = tokio::spawn(async move {
            market.process().await;
        });

        token.cancel();
        match tokio::time::timeout(Duration::from_millis(10), handle).await {
            Err(_) => unreachable!("Should have ended market actor"),
            Ok(_) => {}
        }
    }
}
