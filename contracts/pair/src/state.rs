use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use ura::structs::pair_info::PairInfo;

/// This structure stores the main config parameters for a constant product pair contract.
#[cw_serde]
pub struct Config {
    /// General pair information (e.g pair type)
    pub pair_info: PairInfo,
    /// The factory contract address
    pub factory_addr: Addr,
}

/// Stores the config struct at the given key
pub const CONFIG: Item<Config> = Item::new("config");

/// Keeps track of the lp_token for each lp_provider, this is used to calculate the emission rewards
pub const LP_PROVIDERS: Map<&Addr, Uint128> = Map::new("lp_providers");
