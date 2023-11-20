use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal;
use std::fmt;

use super::asset_info::AssetInfo;
/// ## Description
/// This enum describes a Terra asset (native or CW20).
#[cw_serde]
pub struct DecimalAsset {
    /// Information about an asset stored in a [`AssetInfo`] struct
    pub info: AssetInfo,
    /// A token amount
    pub amount: Decimal,
}

impl fmt::Display for DecimalAsset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.amount, self.info)
    }
}
