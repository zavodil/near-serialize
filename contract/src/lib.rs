// 2 serialization formats within the SDK define how data structures are translated into bytes
// which are needed for passing data into methods of the smart contract or storing data in state:
// - Borsh (https://borsh.io/)
// - JSON (default)

// Borsh Serialization
// A data-structure that can be serialized/deserialized with Borsh and stored in the contract storage in binary format.
// Features:
// - Compact, binary format that's efficient for serialized data size
// - Need to know data format or have a schema to deserialize data
// - Strict and canonical binary representation
// - Fast and less overhead in most cases

// Import Borsh from near_sdk::borsh
use near_sdk::borsh::{self, BorshSerialize, BorshDeserialize};

// JSON Serialization
// Features:
// - Self-describing format (don't need to know the underlying type)
// - Easy interop with JavaScript
// - Less efficient size and (de)serialization

// Import JSON (default) serialization from near_sdk::serde
use near_sdk::serde::{Serialize, Deserialize};

use near_sdk::{AccountId, BorshStorageKey, env, near_bindgen};
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::json_types::U128;

// Define the contract structure
// We read/write data about events, each event belongs to corresponding NEAR account and contains:
// - price [type: Balance] amount on NEAR tokens to pay for event ticket
// - guests [type: UnorderedSet] list of accounts invited to the event
// Event structure defined in the event.rs file

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    events: LookupMap<EventOwnerId, Event>
}

// Define the default, which automatically initializes the contract
impl Default for Contract{
    fn default() -> Self{
        Self{events: LookupMap::new(StorageKey::Events)}
    }
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    // ================= 1 ==================
    // Lets make a method to read event data.

    // Unfortunately, this method doesn't work because Event object contains a UnorderedSet field,
    // which doesn't support JSON serialization

    /* WRONG
    pub fn get_event(&self, event_owner_id: EventOwnerId) -> Event {
        self.internal_get_event(&event_owner_id)
    }
    */

    // In order to mitigate this issue lets create another object EventJSON to properly support
    // JSON output, check event.json.rs file

    // this method works because we converted Event => EventJSON on a fly (event.json.rs#11).
    // We converted Balance => WrappedBalance and UnorderedSet => Vec, to store data in the most
    // efficient and optimized way and output it in a JavaScript friendly format

    // LEGIT
    pub fn get_event(&self, event_owner_id: EventOwnerId) -> EventJSON {
        self
            .internal_get_event(&event_owner_id)// Get Event
            .into() // Convert to EventJSON
    }

    // ================= 2 ==================
    // Lets make a method to write event data.

    // Unfortunately, this method doesn't work too because Event object contains a UnorderedSet field,
    // which doesn't support JSON serialization and we can't provide it as the parameter

    /* WRONG
    pub fn insert_event(&mut self, event: Event) {
        let event_owner_id = env::predecessor_account_id();
        self.events.insert(&event_owner_id, &event);
    }
     */

    // If we provide event as EventJSON, we can parse it and create an Event object. Every guests
    // list has its own UnorderedSet structure initialized by a unique key of BorshStorageKey

    //LEGIT
    pub fn insert_event(&mut self, event: EventJSON) {
        let event_owner_id = env::predecessor_account_id();
        self.events.insert(&event_owner_id.clone(), &Event {
            price: event.price.0,
            guests: UnorderedSet::new(StorageKey::Guests{
                event_owner_id
            })
        });
        self.set_guests(event.guests);
    }

    // helper method to set a list of guests. Again, we can't create a public method and provide
    // UnorderedSet object there

    /* WRONG
    pub fn set_guests(&mut self, guests: UnorderedSet<AccountId>) {
        let mut event = self.internal_get_event(&env::predecessor_account_id());
        event.guests = guests;
        self.internal_set_event(&env::predecessor_account_id(), &event);
    }
     */

    // We can provide a Vec and fill the UnorderedSet object instead
    pub fn set_guests(&mut self, guests: Vec<AccountId>) {
        let mut event = self.internal_get_event(&env::predecessor_account_id());
        for guest in guests {
            event.guests.insert(&guest);
        }
        self.internal_set_event(&env::predecessor_account_id(), &event);
    }

    // And ew can easily use any Borsh object as a parameter in a private method, like this setter:

    // set event helper
    pub(crate) fn internal_set_event(&mut self, event_owner_id: &EventOwnerId, event: &Event) {
        self.events.insert(event_owner_id, event);
    }

    // get event helper
    pub(crate) fn internal_get_event(&self, event_owner_id: &EventOwnerId) -> Event {
        self.events.get(event_owner_id).expect("ERR_MISSING_EVENT")
    }

    // That's pretty much it!
    // Use JSON serialization on input/output if needed and use Borsh serialization to store objects
    // in the contract state.
    // List of available collections: https://docs.rs/near-sdk/latest/near_sdk/collections/#structs
}

/// Helper structure to for keys of the persistent collections.
#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    Events,
    Guests {event_owner_id: EventOwnerId}
}

mod event;
mod event_json;
use event::*;
use event_json::*;

type EventOwnerId = AccountId;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event() {
        let mut contract = Contract::default();

        contract.insert_event(EventJSON {
            price: WrappedBalance::from(1000000000000000000000000),
            guests: vec!(
                AccountId::new_unchecked("alice.testnet".to_string()),
                AccountId::new_unchecked("bob.testnet".to_string())
            )
        });

        let event = contract.get_event(env::predecessor_account_id());

        assert_eq!(event.price.0, 1000000000000000000000000);
        assert_eq!(event.guests.len(), 2);
        assert_eq!(event.guests[0].to_string(), "alice.testnet".to_string());
    }
}
