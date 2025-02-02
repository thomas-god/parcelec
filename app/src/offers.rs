use crate::market::InternalOffer;

pub struct OffersRepository {
    offers: Vec<InternalOffer>,
}
#[derive(Debug, PartialEq)]
pub struct Trade {}

#[derive(Debug, PartialEq)]
pub enum RegisterOfferResult {
    OfferRegistered(String),
    OfferMatched((Trade, Trade)),
}

impl OffersRepository {
    pub fn new() -> OffersRepository {
        OffersRepository { offers: Vec::new() }
    }

    pub fn register_offer(&mut self, new_offer: InternalOffer) -> RegisterOfferResult {
        let offer_id = new_offer.id.clone();
        self.offers.push(new_offer);
        RegisterOfferResult::OfferRegistered(offer_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        market::{Direction, InternalOffer, Offer},
        offers::RegisterOfferResult,
    };

    use super::OffersRepository;

    #[test]
    fn test_register_offer_empty_repository() {
        let mut repository = OffersRepository::new();

        let offer = InternalOffer {
            offer: Offer {
                direction: Direction::Buy,
                volume: 10,
                price: 50_00,
            },
            owner: "toto".to_string(),
            id: "1234".to_string(),
        };

        let res = repository.register_offer(offer);
        assert_eq!(
            res,
            RegisterOfferResult::OfferRegistered("1234".to_string())
        );
    }
}
