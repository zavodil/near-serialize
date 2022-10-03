use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Event {
    pub price: Balance,
    pub guests: UnorderedSet<AccountId>,
}