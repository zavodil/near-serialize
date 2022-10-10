use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct EventJSON {
    pub price: U128,
    pub guests: Vec<AccountId>
}

// method to create EventJSON on a fly
impl From<Event> for EventJSON {
    fn from(event: Event) -> Self {
        EventJSON {
            price: U128::from(event.price),
            guests: event.guests.to_vec()
        }
    }
}
