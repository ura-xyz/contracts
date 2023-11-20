use cosmwasm_schema::cw_serde;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Binary, Decimal, Uint128};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub base_token: String,
    pub epoch_length: u64,
    pub initial_supply: Uint128,
    pub current_epoch: u64,
    pub inflation: Decimal,
    pub decay: Decimal,
    pub min_inflation: Decimal,
    pub team_allocation: Decimal,
    pub team_wallet: Addr,
    // Used in phrase 2 to start emissions
    pub is_emitting: bool,
    pub epoch_start_time: u64,
    pub controller: Addr,
    pub ve_stake: Addr,
}

#[cw_serde]
pub struct Logo {
    pub mime_type: String,
    pub data: Binary,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const LOGO: Item<Logo> = Item::new("logo");
