use super::asset_info::AssetInfo;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal256;

/// This struct describes a Terra asset as decimal.
#[cw_serde]
pub struct Decimal256Asset {
    pub info: AssetInfo,
    pub amount: Decimal256,
}
