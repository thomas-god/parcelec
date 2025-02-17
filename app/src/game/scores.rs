use std::{collections::HashMap, ops::Add};

use serde::Serialize;

use crate::{
    market::{
        models::Direction,
        order_book::{Trade, TradeLeg},
    },
    plants::{PlantId, PlantOutput},
    player::PlayerId,
};

const POSITIVE_IMBALANCE_COST: isize = 50;
const NEGATIVE_IMBALANCE_COST: isize = 100;

#[derive(Debug, PartialEq, Default, Clone, Serialize)]
pub struct PlayerScore {
    pub balance: isize,
    pub pnl: isize,
    pub imbalance_cost: isize,
}

impl Add<PlayerScore> for PlayerScore {
    type Output = PlayerScore;
    fn add(self, rhs: PlayerScore) -> Self::Output {
        PlayerScore {
            balance: self.balance + rhs.balance,
            pnl: self.pnl + rhs.pnl,
            imbalance_cost: self.imbalance_cost + rhs.imbalance_cost,
        }
    }
}

impl Add<&PlantOutput> for PlayerScore {
    type Output = PlayerScore;
    fn add(self, rhs: &PlantOutput) -> PlayerScore {
        PlayerScore {
            balance: self.balance + rhs.setpoint,
            pnl: self.pnl - rhs.cost,
            imbalance_cost: self.imbalance_cost,
        }
    }
}

impl Add<TradeLeg> for PlayerScore {
    type Output = PlayerScore;
    fn add(self, rhs: TradeLeg) -> Self::Output {
        let volume = isize::saturating_add_unsigned(0, rhs.volume);
        let trade_volume = if rhs.direction == Direction::Buy {
            volume
        } else {
            -volume
        };
        let trade_pnl = if rhs.direction == Direction::Buy {
            -rhs.price * volume / 100 // Price in cts
        } else {
            rhs.price * volume / 100 // Price in cts
        };
        PlayerScore {
            balance: self.balance + trade_volume,
            pnl: self.pnl + trade_pnl,
            imbalance_cost: self.imbalance_cost,
        }
    }
}

pub fn compute_players_scores(
    trades: Vec<Trade>,
    plants_outputs: HashMap<PlayerId, HashMap<PlantId, PlantOutput>>,
) -> HashMap<PlayerId, PlayerScore> {
    plants_outputs
        .iter()
        .map(|(player_id, outputs)| {
            (
                player_id.clone(),
                compute_player_score(player_id, outputs, &trades),
            )
        })
        .collect()
}

fn compute_player_score(
    player_id: &PlayerId,
    outputs: &HashMap<PlantId, PlantOutput>,
    trades: &[Trade],
) -> PlayerScore {
    let market_position = trades
        .iter()
        .flat_map(|trade| trade.for_player(player_id))
        .fold(PlayerScore::default(), |acc, trade| acc + trade);
    let plant_position = outputs
        .values()
        .fold(PlayerScore::default(), |acc, output| acc + output);

    let mut player_position = plant_position + market_position;
    player_position.imbalance_cost = match player_position.balance {
        balance if balance > 0 => balance * POSITIVE_IMBALANCE_COST,
        balance if balance < 0 => balance * NEGATIVE_IMBALANCE_COST,
        _ => 0,
    };
    player_position
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::Utc;

    use crate::{
        game::scores::{
            compute_players_scores, PlayerScore, NEGATIVE_IMBALANCE_COST, POSITIVE_IMBALANCE_COST,
        },
        market::order_book::Trade,
        plants::{PlantId, PlantOutput},
        player::PlayerId,
    };

    #[test]
    fn test_scores_no_players() {
        assert_eq!(
            compute_players_scores(Vec::new(), HashMap::new()),
            HashMap::new()
        );
    }

    #[test]
    fn test_scores_no_trades_single_player_imbalanced() {
        let trades = Vec::new();
        let plants_outputs = HashMap::from([(
            PlayerId::from("player_1"),
            HashMap::from([
                (
                    PlantId::from("plant_1"),
                    PlantOutput {
                        setpoint: 100,
                        cost: 100,
                    },
                ),
                (
                    PlantId::from("plant_2"),
                    PlantOutput {
                        setpoint: 200,
                        cost: 500,
                    },
                ),
            ]),
        )]);

        assert_eq!(
            compute_players_scores(trades, plants_outputs),
            HashMap::from([(
                PlayerId::from("player_1"),
                PlayerScore {
                    balance: 300,
                    pnl: -600,
                    imbalance_cost: 300 * POSITIVE_IMBALANCE_COST
                }
            )])
        )
    }

    #[test]
    fn test_scores_with_trades_single_player_single_plant_imbalanced() {
        let trades = Vec::from([Trade {
            buyer: PlayerId::from("player_1"),
            seller: PlayerId::from("another_player"),
            execution_time: Utc::now(),
            price: 80_00,
            volume: 100,
        }]);
        let plants_outputs = HashMap::from([(
            PlayerId::from("player_1"),
            HashMap::from([
                (
                    PlantId::from("plant_1"),
                    PlantOutput {
                        setpoint: 100,
                        cost: 100,
                    },
                ),
                (
                    PlantId::from("plant_2"),
                    PlantOutput {
                        setpoint: 200,
                        cost: 500,
                    },
                ),
            ]),
        )]);

        assert_eq!(
            compute_players_scores(trades, plants_outputs),
            HashMap::from([(
                PlayerId::from("player_1"),
                PlayerScore {
                    balance: 300 + 100,
                    pnl: -600 - (80 * 100),
                    imbalance_cost: 400 * POSITIVE_IMBALANCE_COST
                }
            )])
        )
    }

    #[test]
    fn test_scores_multiple_players() {
        let trades = Vec::from([Trade {
            buyer: PlayerId::from("player_1"),
            seller: PlayerId::from("another_player"),
            execution_time: Utc::now(),
            price: 80_00,
            volume: 100,
        }]);
        let plants_outputs = HashMap::from([
            (
                PlayerId::from("player_1"),
                HashMap::from([
                    (
                        PlantId::from("plant_1"),
                        PlantOutput {
                            setpoint: 100,
                            cost: 100,
                        },
                    ),
                    (
                        PlantId::from("plant_2"),
                        PlantOutput {
                            setpoint: 200,
                            cost: 500,
                        },
                    ),
                ]),
            ),
            (
                PlayerId::from("another_player"),
                HashMap::from([(
                    PlantId::from("another_plant"),
                    PlantOutput {
                        setpoint: -1000,
                        cost: 0,
                    },
                )]),
            ),
        ]);

        assert_eq!(
            compute_players_scores(trades, plants_outputs),
            HashMap::from([
                (
                    PlayerId::from("player_1"),
                    PlayerScore {
                        balance: 300 + 100,
                        pnl: -600 - (80 * 100),
                        imbalance_cost: 400 * POSITIVE_IMBALANCE_COST
                    }
                ),
                (
                    PlayerId::from("another_player"),
                    PlayerScore {
                        balance: -1000 - 100,
                        #[allow(clippy::identity_op)] // Make test more explicit
                        pnl: 0 + (80 * 100),
                        imbalance_cost: -1100 * NEGATIVE_IMBALANCE_COST
                    }
                )
            ])
        )
    }
}
