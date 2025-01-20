use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

#[derive(CandidType, Deserialize)]
struct Message {
    message: String,
}

impl Storable for Message {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

type Memory = VirtualMemory<DefaultMemoryImpl>;
const MAX_VALUE_SIZE: u32 = 100;

impl BoundedStorable for Message {
    const MAX_SIZE: u32 = MAX_VALUE_SIZE;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static MESSAGE_MAP: RefCell<StableBTreeMap<u64, Message, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
        )
    );

}

#[ic_cdk_macros::query]
fn get_message(key: u64) -> Option<Message> {
    MESSAGE_MAP.with(|p| p.borrow().get(&key))
}

#[ic_cdk_macros::update]
fn create_message(key: u64, message: String) -> Option<Message> {
    let value = Message { message };
    // MESSAGE_MAP
    let does_exist: bool = MESSAGE_MAP.with(|m| m.borrow().contains_key(&key));
    if does_exist {
        return None;
    }
    MESSAGE_MAP.with(|p| p.borrow_mut().insert(key, value))
}

#[ic_cdk_macros::update]
fn update_message(key: u64, message: String) -> Option<Message> {
    let value = Message { message };
    MESSAGE_MAP.with(|p| p.borrow_mut().insert(key, value))
}

#[ic_cdk_macros::update]
fn delete_message(key: u64) -> Option<Message> {
    MESSAGE_MAP.with(|p| p.borrow_mut().remove(&key))
}

#[ic_cdk_macros::query]
fn get_all_messages() -> Option<Vec<(u64, Message)>> {
    MESSAGE_MAP.with(|map| {
        let map = map.borrow();
        let messages: Vec<(u64, Message)> = map.iter()
            .collect();
        if messages.is_empty() {
            None
        } else {
            Some(messages)
        }
    })
}

