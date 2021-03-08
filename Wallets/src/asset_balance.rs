use neo_core::fixed8::fixed8;
use crate::coin::Coin;
use std::error::Error;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, Serialize, Deserialize)]
pub struct AssetBalance {
    balance: fixed8,
    unspent: Vec(Coin),
    spent: Vec(Coin),
    unconfirmed: Vec(Coin),
}


/**
 * Balance of an UTXO asset.
 * We keep track of 3 states: unspent, spent and unconfirmed.
 * Unspent coins are ready to be constructed into transactions.
 * Spent coins have been used once in confirmed transactions and cannot be used anymore. They are kept here for tracking purposes.
 * Unconfirmed coins have been used in transactions but are not confirmed yet. self is a holding state until we confirm that the transactions are mined into blocks.
 */
impl AssetBalance {

    pub fn unspent<'a>(&self) -> &Vec<(Coin<'a>), Error> {
        &self.unspent
    }
    pub fn spent<'a>(&self) -> &Vec<(Coin<'a>), Error> {
        &self.spent
    }
    pub fn unconfirmed<'a>(&self) -> &Vec<(Coin<'a>), Error> {
        &self.unconfirmed
    }

    pub fn get_balance(&self) -> fixed8 {
        self.unspent
            .iter()
            .map(|p: &Coin| c.value())
            .fold(fixed8(0), |acc, len| acc + len)
    }


    pub fn export(&self) -> AssetBalance {
        AssetBalance {
            balance: self.balance.clone(),
            unspent: self.unspent.clone(),
            spent: self.spent.clone(),
            unconfirmed: self.unconfirmed.clone(),
        }
    }

    pub fn equals(&self, other: &AssetBalance) -> bool {
        self.unspent.equals(&other.unspent) &&
            self.spent.equals(&other.spent) &&
            self.unconfirmed.equals(&other.unconfirmed)
    }
}