use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, DepsMut, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use ura::structs::asset_info::AssetInfo;
use ura::structs::pair_info::PairInfo;

/// This structure stores the main stableswap pair parameters.
#[cw_serde]
pub struct Config {
    /// The pair information stored in a [`PairInfo`] struct
    pub pair_info: PairInfo,
    /// The factory contract address
    pub factory_addr: Addr,
    /// This is the current amplification used in the pool
    pub init_amp: u64,
    /// This is the start time when amplification starts to scale up or down
    pub init_amp_time: u64,
    /// This is the target amplification to reach at `next_amp_time`
    pub next_amp: u64,
    /// This is the timestamp when the current pool amplification should be `next_amp`
    pub next_amp_time: u64,
    /// The greatest precision of assets in the pool
    pub greatest_precision: u8,
}

pub const CONFIG: Item<Config> = Item::new("config");

/// Stores map of AssetInfo (as String) -> precision
pub const PRECISIONS: Map<String, u8> = Map::new("precisions");

/// Store all token precisions and return the greatest one.
pub(crate) fn store_precisions(
    deps: DepsMut,
    asset_infos: &[AssetInfo],
    factory_addr: &Addr,
) -> StdResult<u8> {
    let mut max = 0u8;

    for asset_info in asset_infos {
        let precision = asset_info.decimals(&deps.querier, factory_addr)?;
        max = max.max(precision);
        PRECISIONS.save(deps.storage, asset_info.to_string(), &precision)?;
    }

    Ok(max)
}

/// Loads precision of the given asset info.
pub(crate) fn get_precision(storage: &dyn Storage, asset_info: &AssetInfo) -> StdResult<u8> {
    PRECISIONS.load(storage, asset_info.to_string())
}

/// Keeps track of the lp_token for each lp_provider, this is used to calculate the emission rewards
pub const LP_PROVIDERS: Map<&Addr, Uint128> = Map::new("lp_providers");
