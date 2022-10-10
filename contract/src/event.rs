use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Event {
    pub price: u128,
    pub guests: UnorderedSet<AccountId>,
}