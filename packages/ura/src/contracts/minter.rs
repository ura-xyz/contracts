use crate::contracts::token::InstantiateMarketingInfo;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Decimal, Uint128};
use cw20::Cw20Coin;

#[cw_serde]
pub struct InstantiateMsg {
    pub epoch_start_time: u64,
    pub epoch_duration: u64,
    pub initial_supply: Uint128,
    pub inflation: Decimal,
    pub decay: Decimal,
    pub min_inflation: Decimal,
    pub team_allocation: Decimal,
    pub team_wallet: String,
    pub base_token_params: BaseTokenParams,
}

#[cw_serde]
pub enum ExecuteMsg {
    EndEpoch {},
    SetVeStaking {},
    SetGaugeController {},
    UpdateConfig(UpdateConfigRequest),
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Binary)]
    Config {},
    /// Returns metadata on the contract - name, decimals, supply, etc.
    /// Return type: TokenInfoResponse.
    #[returns(TokenInfoResponse)]
    TokenInfo {},
    /// Only with "marketing" extension
    /// Downloads the embedded logo data (if stored on chain). Errors if no logo data stored for
    /// this contract.
    /// Return type: DownloadLogoResponse.
    #[returns(DownloadLogoResponse)]
    DownloadLogo {},
}

#[cw_serde]
pub struct BaseTokenParams {
    pub marketing_info: Option<InstantiateMarketingInfo>,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<Cw20Coin>,
}

#[cw_serde]
pub struct UpdateConfigRequest {
    pub epoch_length: Option<u64>,
    pub inflation: Option<Decimal>,
    pub decay: Option<Decimal>,
    pub min_inflation: Option<Decimal>,
    pub team_allocation: Option<Decimal>,
    pub team_wallet: Option<String>,
    pub is_emitting: Option<bool>,
    pub epoch_start_time: Option<u64>,
}

#[cw_serde]
pub struct TokenInfoResponse {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
}

#[cw_serde]
pub struct DownloadLogoResponse {
    pub mime_type: String,
    pub data: Binary,
}
