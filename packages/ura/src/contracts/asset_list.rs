use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::structs::{asset::Asset, asset_info::AssetInfo};

#[cw_serde]
pub struct InstantiateMsg {
    pub admins: Vec<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddAssetMetadata(AssetMetadata),
    RemoveAssetMetadata(AssetInfo),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetAssetsResponse)]
    GetAssets(AssetInfoPagination),
    #[returns(AssetMetadata)]
    GetAsset(AssetInfo),
    #[returns(GetBalancesResponse)]
    GetBalances {
        address: String,
        pagination: AssetInfoPagination,
    },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct AssetMetadata {
    pub info: AssetInfo,
    pub symbol: String,
    pub name: String,
    pub icon: String,
    pub decimals: u64,
}

#[cw_serde]
pub struct AssetInfoPagination {
    pub limit: u64,
    pub last: Option<AssetInfo>,
}

#[cw_serde]
pub struct GetAssetsResponse {
    pub assets: Vec<AssetMetadata>,
}

#[cw_serde]
pub struct GetBalancesResponse {
    pub balances: Vec<Asset>,
}
