use cosmwasm_std::{Addr, Empty};
use cw_storage_plus::{Item, Map};

/// Owner of the xp contract to be able to add and remove whitelisted addresses
pub const OWNER: Item<Addr> = Item::new("owner");

/// Keeps track of the addresses that can interact with the contract
pub const WHITELISTED_ADDRESS: Map<&Addr, Empty> = Map::new("whitelisted_address");
