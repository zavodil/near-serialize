# Borsh and JSON serialization on NEAR 

There are 2 serialization formats within the SDK to define how data structures are translated into bytes:

- Borsh (https://borsh.io/) 
- JSON (default)

## Borsh Serialization

A data-structure can be serialized/deserialized with Borsh to **store in the contract storage in binary format**.
 
Features:
  - Compact, binary format that's efficient for serialized data size
  - Need to know data format or have a schema to deserialize data
  - Strict and canonical binary representation
  - Fast and less overhead in most cases

## JSON Serialization

A data-structure serialized/deserialized with JSON is using for passing data into input/output methods of the smart contract.

Features:
 - Self-describing format (don't need to know the underlying type)
 - Easy interop with JavaScript
 - Less efficient size and (de)serialization


## How to use Borsh Serialization

Lets make an example application to store events, each of which has owner, ticket price and guests list.

### Import Borsh
```rust
use near_sdk::borsh::{self, BorshSerialize, BorshDeserialize};
```

### Define the contract structure

We read/write data about events, each event belongs to corresponding NEAR account and contains:
 - price [type: u128] amount on NEAR tokens to pay for event ticket
 - guests [type: UnorderedSet] list of accounts invited to the event

[Full contract code](contract/src/lib.rs).

```rust
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    events: LookupMap<EventOwnerId, Event>
}
```

### Define a data-structure for `Event` object

```rust
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Event {
    pub price: u128,
    pub guests: UnorderedSet<AccountId>,
}
```

Create a helper structure to for keys of the persistent collections.

```rust
#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    Events,
    Guests {event_owner_id: EventOwnerId}
}
```

Define the default, which automatically initializes the contract

```rust
impl Default for Contract{
    fn default() -> Self{
        Self{
            events: LookupMap::new(StorageKey::Events)
        }
    }
}
```

## Read method

Lets make a method to read event data.

```rust
    // WRONG
    pub fn get_event(&self, event_owner_id: EventOwnerId) -> Event {
        self.events.get(&event_owner_id).expect("ERR_MISSING_EVENT")
    }
```

Unfortunately, this method doesn't work because `Event` object contains a `UnorderedSet` field,
which doesn't support JSON serialization and we can't output it.

In order to mitigate this issue lets create another object `EventJSON` to properly support
JSON output.

```rust
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct EventJSON {
    pub price: U128,
    pub guests: Vec<AccountId>
}
```

`U128` is a json-friendly type for u128, [more info](https://docs.rs/near-sdk/latest/near_sdk/json_types/struct.U128.html).

Let's implement a method to convert `Event` to `EventJSON` object

```rust 
impl From<Event> for EventJSON {
    fn from(event: Event) -> Self {
        EventJSON {
            price: U128::from(event.price),
            guests: event.guests.to_vec()
        }
    }
}
```

We converted `u128` => `U128` and `UnorderedSet` => `Vec`, because we want to store data in the most 
efficient and optimized way (`u128`, `UnorderedSet`) and output in a JavaScript friendly format (`U128` to send big 
numbers in string format and avoid number overflow and `Vec` to enable simple array)

Now we can use `EventJSON` object to create a `get_event` method.

```rust
  pub fn get_event(&self, event_owner_id: EventOwnerId) -> EventJSON {
        self.events
            .get(&event_owner_id).expect("ERR_MISSING_EVENT") // Get Event
            .into() // Convert to EventJSON
    }
```

This method works because we converted Event => EventJSON on a fly.

## Write method

Let's make a method to write event data.

```rust
    // WRONG 
    pub fn insert_event(&mut self, event: Event) {
        let event_owner_id = env::predecessor_account_id();
        self.events.insert(&event_owner_id, &event);
    }
```

Unfortunately, this method doesn't work too because `Event` object contains a `UnorderedSet` field, which doesn't support 
JSON serialization and we can't provide it as the parameter.

If we provide event as `EventJSON`, we can parse it and create an `Event` object. 
Every `guests` list has its own `UnorderedSet` structure initialized by a unique key of `BorshStorageKey` collection.

```rust    
    pub fn insert_event(&mut self, event: EventJSON) {
        let event_owner_id = env::predecessor_account_id();
        self.events.insert(&event_owner_id.clone(), &Event {
            price: event.price.0,
            guests: UnorderedSet::new(StorageKey::Guests {
                event_owner_id
            })
        });
        self.set_guests(event.guests);
    }
```

## A few more examples

Let's create a helper method to set a list of guests for a given event. 
Again, we can't create a public method and provide `UnorderedSet` object there.

```rust
    //WRONG
    pub fn set_guests(&mut self, guests: UnorderedSet<AccountId>) {
        let mut event = self.internal_get_event(&env::predecessor_account_id());
        event.guests = guests;
        self.internal_set_event(&env::predecessor_account_id(), &event);
    }
   ```
     
We can provide a `Vec` and fill the `UnorderedSet` object instead
```rust
    pub fn set_guests(&mut self, guests: Vec<AccountId>) {
        let mut event = self.internal_get_event(&env::predecessor_account_id());
        for guest in guests {
            event.guests.insert(&guest);
        }
        self.internal_set_event(&env::predecessor_account_id(), &event);
    }
```

And we can easily use any Borsh object as a parameter in a private method, like with this setter:

```rust
    pub(crate) fn internal_set_event(&mut self, event_owner_id: &EventOwnerId, event: &Event) {
        self.events.insert(event_owner_id, event);
    }
```    

---

That's pretty much it!

> Use `JSON` serialization on input/output if needed and use `Borsh` serialization to store objects
in the contract state. You will save gas for internal operations and reduce expenses for a storage staking then.

### Additional links

- [Contract code of this tutorial](contract/src/lib.rs)
- [List of available collections](https://docs.rs/near-sdk/latest/near_sdk/collections/#structs)
- [Docs portal](https://docs.near.org)
  
