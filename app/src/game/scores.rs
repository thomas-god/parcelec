use std::{collections::HashMap, fmt::Debug, ops::Add};

use serde::Serialize;

use crate::{
    constants::{NEGATIVE_IMBALANCE_COST, POSITIVE_IMBALANCE_COST},
    market::{
        Direction,
        order_book::{Trade, TradeLeg},
    },
    plants::{PlantId, PlantOutput},
    player::PlayerId,
    utils::units::{EnergyCost, Money, NO_POWER, Power, TIMESTEP},
};

use super::delivery_period::DeliveryPeriodId;

#[derive(Debug, PartialEq, Default, Clone, Serialize)]
pub struct PlayerScore {
    pub balance: Power,
    pub pnl: isize,
    pub imbalance_cost: Money,
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
            balance: self.balance + trade_volume.into(),
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
        balance if balance > NO_POWER => {
            balance * TIMESTEP * EnergyCost::from(POSITIVE_IMBALANCE_COST)
        }
        balance if balance < NO_POWER => {
            balance * TIMESTEP * EnergyCost::from(NEGATIVE_IMBALANCE_COST)
        }
        _ => 0.into(),
    };
    player_position
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::Utc;

    use crate::{
        game::scores::{
            NEGATIVE_IMBALANCE_COST, POSITIVE_IMBALANCE_COST, PlayerScore, compute_players_scores,
        },
        market::order_book::Trade,
        plants::{PlantId, PlantOutput},
        player::PlayerId,
        utils::units::{Money, Power},
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
                        setpoint: Power::from(100),
                        cost: 100,
                    },
                ),
                (
                    PlantId::from("plant_2"),
                    PlantOutput {
                        setpoint: Power::from(200),
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
                    balance: Power::from(300),
                    pnl: -600,
                    imbalance_cost: Money::from(300 * POSITIVE_IMBALANCE_COST)
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
                        setpoint: Power::from(100),
                        cost: 100,
                    },
                ),
                (
                    PlantId::from("plant_2"),
                    PlantOutput {
                        setpoint: Power::from(200),
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
                    balance: Power::from(300 + 100),
                    pnl: -600 - (80 * 100),
                    imbalance_cost: Money::from(400 * POSITIVE_IMBALANCE_COST)
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
                            setpoint: Power::from(100),
                            cost: 100,
                        },
                    ),
                    (
                        PlantId::from("plant_2"),
                        PlantOutput {
                            setpoint: Power::from(200),
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
                        setpoint: Power::from(-1000),
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
                        balance: Power::from(300 + 100),
                        pnl: -600 - (80 * 100),
                        imbalance_cost: Money::from(400 * POSITIVE_IMBALANCE_COST)
                    }
                ),
                (
                    PlayerId::from("another_player"),
                    PlayerScore {
                        balance: Power::from(-1000 - 100),
                        #[allow(clippy::identity_op)] // Make test more explicit
                        pnl: 0 + (80 * 100),
                        imbalance_cost: Money::from(-1100 * NEGATIVE_IMBALANCE_COST)
                    }
                )
            ])
        )
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct PlayerResult {
    pub player: PlayerId,
    pub rank: usize,
    pub score: isize,
    pub tier: Option<RankTier>,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum RankTier {
    Bronze,
    Silver,
    Gold,
}

#[derive(Debug, Clone)]
pub struct TierLimits {
    pub gold: isize,
    pub silver: isize,
    pub bronze: isize,
}

#[derive(Debug, Clone)]
pub struct GameRankings {
    pub tier_limits: Option<TierLimits>,
}

impl GameRankings {
    pub fn compute_scores(
        &self,
        players_scores: &HashMap<PlayerId, HashMap<DeliveryPeriodId, PlayerScore>>,
    ) -> Vec<PlayerResult> {
        let mut scores: Vec<(PlayerId, isize)> = players_scores
            .iter()
            .map(|(player, score)| {
                (
                    player.clone(),
                    score
                        .iter()
                        .fold(0, |acc, (_, s)| acc + s.pnl + isize::from(s.imbalance_cost)),
                )
            })
            .collect();
        scores.sort_by(|(_, a), (_, b)| b.cmp(a));
        scores
            .iter()
            .enumerate()
            .map(|(idx, (player_id, score))| PlayerResult {
                player: player_id.clone(),
                rank: idx + 1,
                score: *score,
                tier: match (&self.tier_limits, score) {
                    (None, _) => None,
                    (Some(limits), score) => match score {
                        score if *score >= limits.gold => Some(RankTier::Gold),
                        score if *score >= limits.silver => Some(RankTier::Silver),
                        score if *score >= limits.bronze => Some(RankTier::Bronze),
                        _ => None,
                    },
                },
            })
            .collect()
    }
}

#[cfg(test)]
mod test_pnl_ranking {
    use std::collections::HashMap;

    use crate::{
        game::{
            delivery_period::DeliveryPeriodId,
            scores::{GameRankings, PlayerResult, RankTier, TierLimits},
        },
        player::PlayerId,
        utils::units::{Money, Power},
    };

    use super::PlayerScore;

    #[test]
    fn test_pnl_ranking_no_tiers() {
        let scores = HashMap::from([
            (
                PlayerId::from("toto"),
                HashMap::from([
                    (
                        DeliveryPeriodId::from(1),
                        PlayerScore {
                            balance: Power::from(0),
                            imbalance_cost: Money::from(0),
                            pnl: 1000,
                        },
                    ),
                    (
                        DeliveryPeriodId::from(2),
                        PlayerScore {
                            balance: Power::from(10),
                            imbalance_cost: Money::from(-60),
                            pnl: 1050,
                        },
                    ),
                ]),
            ),
            (
                PlayerId::from("other_player"),
                HashMap::from([
                    (
                        DeliveryPeriodId::from(1),
                        PlayerScore {
                            balance: Power::from(0),
                            imbalance_cost: Money::from(0),
                            pnl: 0,
                        },
                    ),
                    (
                        DeliveryPeriodId::from(2),
                        PlayerScore {
                            balance: Power::from(10),
                            imbalance_cost: Money::from(-60),
                            pnl: 0,
                        },
                    ),
                ]),
            ),
        ]);

        let ranking = GameRankings { tier_limits: None };
        assert_eq!(
            ranking.compute_scores(&scores),
            vec![
                PlayerResult {
                    player: PlayerId::from("toto"),
                    rank: 1,
                    score: 1990,
                    tier: None
                },
                PlayerResult {
                    player: PlayerId::from("other_player"),
                    rank: 2,
                    score: -60,
                    tier: None
                }
            ]
        )
    }

    #[test]
    fn test_pnl_ranking_with_tiers() {
        let scores = HashMap::from([
            (
                PlayerId::from("gold"),
                HashMap::from([(
                    DeliveryPeriodId::from(1),
                    PlayerScore {
                        balance: Power::from(0),
                        imbalance_cost: Money::from(0),
                        pnl: 1000,
                    },
                )]),
            ),
            (
                PlayerId::from("silver"),
                HashMap::from([(
                    DeliveryPeriodId::from(1),
                    PlayerScore {
                        balance: Power::from(0),
                        imbalance_cost: Money::from(0),
                        pnl: 500,
                    },
                )]),
            ),
            (
                PlayerId::from("bronze"),
                HashMap::from([(
                    DeliveryPeriodId::from(1),
                    PlayerScore {
                        balance: Power::from(0),
                        imbalance_cost: Money::from(0),
                        pnl: 100,
                    },
                )]),
            ),
            (
                PlayerId::from("none"),
                HashMap::from([(
                    DeliveryPeriodId::from(1),
                    PlayerScore {
                        balance: Power::from(0),
                        imbalance_cost: Money::from(0),
                        pnl: -500,
                    },
                )]),
            ),
        ]);

        let ranking = GameRankings {
            tier_limits: Some(TierLimits {
                gold: 1000,
                silver: 500,
                bronze: 100,
            }),
        };
        assert_eq!(
            ranking.compute_scores(&scores),
            vec![
                PlayerResult {
                    player: PlayerId::from("gold"),
                    rank: 1,
                    score: 1000,
                    tier: Some(RankTier::Gold)
                },
                PlayerResult {
                    player: PlayerId::from("silver"),
                    rank: 2,
                    score: 500,
                    tier: Some(RankTier::Silver)
                },
                PlayerResult {
                    player: PlayerId::from("bronze"),
                    rank: 3,
                    score: 100,
                    tier: Some(RankTier::Bronze)
                },
                PlayerResult {
                    player: PlayerId::from("none"),
                    rank: 4,
                    score: -500,
                    tier: None
                }
            ]
        )
    }
}
