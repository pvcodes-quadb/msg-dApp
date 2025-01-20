use ic_cdk::{export_candid, query, storage, update};
use std::cell::RefCell;

#[derive(Clone)]
struct MessageEntry {
    content: String,
    timestamp: u64,
    author: String,
}

thread_local! {
    static MESSAGE_STORE: RefCell<Vec<MessageEntry>> = RefCell::new(Vec::new());
}

#[update]
fn store_message(text: String, author: String) -> u64 {
    let timestamp = ic_cdk::api::time();
    let entry = MessageEntry {
        content: text,
        timestamp,
        author,
    };

    MESSAGE_STORE.with(|store| {
        let mut store = store.borrow_mut();
        store.push(entry);
        storage::stable_save((store.clone(),)).unwrap_or_else(|_| panic!("Storage failure"));
        (store.len() - 1) as u64
    })
}

#[query]
fn retrieve_messages() -> Vec<(String, u64, String)> {
    MESSAGE_STORE.with(|store| {
        store
            .borrow()
            .iter()
            .map(|entry| (entry.content.clone(), entry.timestamp, entry.author.clone()))
            .collect()
    })
}

// Added filter by author functionality
#[query]
fn get_messages_by_author(author: String) -> Vec<(String, u64)> {
    MESSAGE_STORE.with(|store| {
        store
            .borrow()
            .iter()
            .filter(|entry| entry.author == author)
            .map(|entry| (entry.content.clone(), entry.timestamp))
            .collect()
    })
}

#[update]
fn update_message(
    position: usize,
    updated_text: String,
    author: String,
) -> Result<(), &'static str> {
    MESSAGE_STORE.with(|store| {
        let mut store = store.borrow_mut();
        if position >= store.len() {
            return Err("Invalid message position");
        }

        // Only allow updates if the author matches
        if store[position].author != author {
            return Err("Unauthorized: only the author can modify the message");
        }

        store[position].content = updated_text;
        store[position].timestamp = ic_cdk::api::time(); // Update timestamp on modification

        storage::stable_save((store.clone(),)).unwrap_or_else(|_| panic!("Storage failure"));
        Ok(())
    })
}

#[update]
fn remove_message(position: usize, author: String) -> Result<(), &'static str> {
    MESSAGE_STORE.with(|store| {
        let mut store = store.borrow_mut();
        if position >= store.len() {
            return Err("Invalid message position");
        }

        // Only allow deletion if the author matches
        if store[position].author != author {
            return Err("Unauthorized: only the author can delete the message");
        }

        store.remove(position);
        storage::stable_save((store.clone(),)).unwrap_or_else(|_| panic!("Storage failure"));
        Ok(())
    })
}

// Added new function to get message count
#[query]
fn get_message_count() -> usize {
    MESSAGE_STORE.with(|store| store.borrow().len())
}

export_candid!();
