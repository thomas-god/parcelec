use std::{collections::HashMap, ops::Add};

use serde::Serialize;

use crate::{
    market::{
        models::Direction,
        order_book::{Trade, TradeLeg},
    },
    plants::PlantOutput,
};

#[derive(Debug, PartialEq, Default, Clone, Serialize)]
pub struct PlayerScore {
    pub balance: isize,
    pub pnl: isize,
}

impl Add<PlayerScore> for PlayerScore {
    type Output = PlayerScore;
    fn add(self, rhs: PlayerScore) -> Self::Output {
        PlayerScore {
            balance: self.balance + rhs.balance,
            pnl: self.pnl + rhs.pnl,
        }
    }
}

impl Add<&PlantOutput> for PlayerScore {
    type Output = PlayerScore;
    fn add(self, rhs: &PlantOutput) -> PlayerScore {
        PlayerScore {
            balance: self.balance + rhs.setpoint,
            pnl: self.pnl - rhs.cost,
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
        }
    }
}

pub fn compute_players_scores(
    trades: Vec<Trade>,
    plants_outputs: HashMap<String, HashMap<String, PlantOutput>>,
) -> HashMap<String, PlayerScore> {
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
    player_id: &str,
    outputs: &HashMap<String, PlantOutput>,
    trades: &[Trade],
) -> PlayerScore {
    let market_position = trades
        .iter()
        .flat_map(|trade| trade.for_player(player_id))
        .fold(PlayerScore::default(), |acc, trade| acc + trade);
    let plant_position = outputs
        .values()
        .fold(PlayerScore::default(), |acc, output| acc + output);

    plant_position + market_position
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::Utc;

    use crate::{
        game::scores::{compute_players_scores, PlayerScore},
        market::order_book::Trade,
        plants::PlantOutput,
    };

    #[test]
    fn test_scores_no_players() {
        assert_eq!(
            compute_players_scores(Vec::new(), HashMap::new()),
            HashMap::new()
        );
    }

    #[test]
    fn test_scores_no_trades_single_player_single_plant() {
        let trades = Vec::new();
        let plants_outputs = HashMap::from([(
            "player_1".to_string(),
            HashMap::from([
                (
                    "plant_1".to_string(),
                    PlantOutput {
                        setpoint: 100,
                        cost: 100,
                    },
                ),
                (
                    "plant_2".to_string(),
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
                "player_1".to_string(),
                PlayerScore {
                    balance: 300,
                    pnl: -600
                }
            )])
        )
    }

    #[test]
    fn test_scores_with_trades_single_player_single_plant() {
        let trades = Vec::from([Trade {
            buyer: "player_1".to_string(),
            seller: "another_player".to_string(),
            execution_time: Utc::now(),
            price: 80_00,
            volume: 100,
        }]);
        let plants_outputs = HashMap::from([(
            "player_1".to_string(),
            HashMap::from([
                (
                    "plant_1".to_string(),
                    PlantOutput {
                        setpoint: 100,
                        cost: 100,
                    },
                ),
                (
                    "plant_2".to_string(),
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
                "player_1".to_string(),
                PlayerScore {
                    balance: 300 + 100,
                    pnl: -600 - (80 * 100)
                }
            )])
        )
    }

    #[test]
    fn test_scores_multiple_players() {
        let trades = Vec::from([Trade {
            buyer: "player_1".to_string(),
            seller: "another_player".to_string(),
            execution_time: Utc::now(),
            price: 80_00,
            volume: 100,
        }]);
        let plants_outputs = HashMap::from([
            (
                "player_1".to_string(),
                HashMap::from([
                    (
                        "plant_1".to_string(),
                        PlantOutput {
                            setpoint: 100,
                            cost: 100,
                        },
                    ),
                    (
                        "plant_2".to_string(),
                        PlantOutput {
                            setpoint: 200,
                            cost: 500,
                        },
                    ),
                ]),
            ),
            (
                "another_player".to_string(),
                HashMap::from([(
                    "another_plant".to_string(),
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
                    "player_1".to_string(),
                    PlayerScore {
                        balance: 300 + 100,
                        pnl: -600 - (80 * 100)
                    }
                ),
                (
                    "another_player".to_string(),
                    PlayerScore {
                        balance: -1000 - 100,
                        #[allow(clippy::identity_op)] // Make test more explicit
                        pnl: 0 + (80 * 100)
                    }
                )
            ])
        )
    }
}
