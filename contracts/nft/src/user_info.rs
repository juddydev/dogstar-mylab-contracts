use crate::error::MyLabError;
use crate::nft_info::read_nft;
use crate::storage_types::{DataKey, Level, TokenId, User};
use soroban_sdk::{log, Address, Env, Vec};

pub fn add_card_to_owner(env: &Env, token_id: TokenId, user: Address) -> Result<(), MyLabError> {
    log!(&env, "Add card to owner function");
    if let Some(card) = read_nft(&env, user.clone(), token_id.clone()) {
        log!(&env, "add_card_to_owner >> Found card {}", card.clone());
        let mut user_card_ids = read_owner_card(&env, user.clone());
        log!(
            &env,
            "add_card_to_owner >> User card ids {}",
            user_card_ids.clone()
        );
        user_card_ids.push_back(token_id.clone());
        write_owner_card(&env, user.clone(), user_card_ids);
        Ok(())
    } else {
        log!(
            &env,
            "add_card_to_owner >> Card not found in add_card_to_owner"
        );
        return Err(MyLabError::NotNFT);
    }
}

// TODO:
// read_user shoudld either return a user or return an error of not found

pub fn read_user(e: &Env, user: Address) -> User {
    let key = DataKey::User(user.clone());
    e.storage().persistent().get(&key).unwrap_or(User {
        owner: user,
        power: 0,
        terry: 0,
        total_history_terry: 0,
        level: 1,
    })
}

pub fn write_user(e: &Env, user: Address, user_info: User) {
    let key = DataKey::User(user);
    e.storage().persistent().set(&key, &user_info);
}

pub fn get_user_level(e: &Env, user: Address) -> u32 {
    let user = read_user(&e, user.clone());
    let balance = user.total_history_terry;
    log!(&e, "get_user_level >> User balance {}", balance);

    // Fetch the last level ID from storage
    let last_level_id = e
        .storage()
        .persistent()
        .get(&DataKey::LevelId)
        .unwrap_or(0u32);

    for i in 1..=last_level_id {
        let level: Level = e.storage().persistent().get(&DataKey::Level(i)).unwrap();
        if balance > level.minimum_terry && balance <= level.maximum_terry {
            return i;
        }
    }

    // Default level if no matching level is found
    1
}

pub fn write_owner_card(env: &Env, owner: Address, token_ids: Vec<TokenId>) {
    log!(
        &env,
        "write_owner_card >> Write owner card for {}, token_ids {}",
        owner.clone(),
        token_ids.clone()
    );
    let key = DataKey::OwnerOwnedCardIds(owner);
    env.storage().persistent().set(&key, &token_ids);
}

pub fn read_owner_card(env: &Env, owner: Address) -> Vec<TokenId> {
    log!(
        &env,
        "read_owner_card >> Read owner card for {}",
        owner.clone()
    );
    let key = DataKey::OwnerOwnedCardIds(owner.clone());

    if !env.storage().persistent().has(&key) {
        log!(&env, "Not found cards for owner {}", owner.clone());
        let empty_vec: Vec<TokenId> = Vec::new(&env);
        env.storage().persistent().set(&key, &empty_vec);
    }

    let card_list: Vec<TokenId> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(&env));

    return card_list;
}

pub fn mint_terry(e: &Env, owner: Address, amount: i128) {
    let mut user = read_user(e, owner.clone());
    user.terry += amount;
    user.total_history_terry += amount;
    user.level = get_user_level(e, owner.clone());
    write_user(e, user.owner.clone(), user);
}

pub fn burn_terry(e: &Env, owner: Address, amount: i128) {
    let mut user = read_user(e, owner.clone());
    if user.terry < amount {
        log!(&e, "burn_terry >> Not enough terry to burn");
        return;
    }
    user.terry -= amount;
    write_user(e, user.owner.clone(), user);
    log!(&e, "burn_terry >> Burned terry {}", amount);
}
