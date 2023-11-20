use cosmwasm_std::{CustomQuery, QuerierWrapper, StdResult, Uint128};
use itertools::Itertools;

use super::querier::query_token_symbol;
use crate::structs::asset_info::AssetInfo;

/// UST token denomination
pub const UUSD_DENOM: &str = "uusd";
/// LUNA token denomination
pub const ULUNA_DENOM: &str = "uluna";
/// Minimum initial LP share
pub const MINIMUM_LIQUIDITY_AMOUNT: Uint128 = Uint128::new(1_000);
/// Maximum denom length
pub const DENOM_MAX_LENGTH: usize = 128;

const TOKEN_SYMBOL_MAX_LENGTH: usize = 4;

/// Returns a formatted LP token name
pub fn format_lp_token_name<C>(
    asset_infos: &[AssetInfo],
    querier: &QuerierWrapper<C>,
) -> StdResult<String>
where
    C: CustomQuery,
{
    let mut short_symbols: Vec<String> = vec![];
    for asset_info in asset_infos {
        let short_symbol = match &asset_info {
            AssetInfo::NativeToken { denom } => {
                denom.chars().take(TOKEN_SYMBOL_MAX_LENGTH).collect()
            }
            AssetInfo::Token { contract_addr } => {
                let token_symbol = query_token_symbol(querier, contract_addr)?;
                token_symbol.chars().take(TOKEN_SYMBOL_MAX_LENGTH).collect()
            }
        };
        short_symbols.push(short_symbol);
    }
    Ok(format!("{}-LP", short_symbols.iter().join("-")).to_uppercase())
}
