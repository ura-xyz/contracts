use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20ReceiveMsg;
use std::collections::HashMap;

use crate::structs::asset::Asset;

#[cw_serde]
pub struct InstantiateMsg {
    pub epoch_start_time: u64,
    pub epoch_duration: u64,
    pub gauge_code_id: u64,
    pub asset_list: String,
    pub factory_addr: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateGauge(Pool),
    DepositBribe(Pool),
    ClaimGaugeRewards(Pool),
    Vote(Votes),
    EndEpoch(EndEpochRequest),
    Receive(Cw20ReceiveMsg),
    SetVeStaking {},
    SetEmissionToken(SetEmissionTokenRequest),
    ClaimEmissionRewards(Pool),
    UpdateEmissions(UpdateEmissionsRequest),
    AccumUserEmissions(AccumEmissionsRequest),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(UserVotesResponse)]
    UserVotes(UserVotesRequest),
    #[returns(TotalGaugeVotesResponse)]
    TotalGaugeVotes(TotalGaugeVotesRequest),
    #[returns(UserGaugeVotesResponse)]
    UserGaugeVotes(UserGaugeVotesRequest),
    #[returns(AllGaugesResponse)]
    AllGauges(AllGaugesRequest),
    #[returns(GaugeFromPoolResponse)]
    GaugeFromPool(GaugeFromPoolRequest),
    #[returns(EmissionRewardsResponse)]
    EmissionRewards(EmissionRewardsRequest),
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum Cw20HookMsg {
    DepositBribe(Pool),
}

#[cw_serde]
pub struct Votes {
    pub weights: Vec<Uint128>,
    pub gauges: Vec<String>,
}

#[cw_serde]
pub struct Pool {
    pub pool: String,
}

#[cw_serde]
pub struct ClaimAllPoolFeesRequest {
    pub limit: u64,
    pub last_pool: Option<String>,
}

#[cw_serde]
pub struct WithdrawPoolTokensRequest {
    pub token: String,
    pub amount: Option<Uint128>,
    pub to_address: Option<String>,
}

#[cw_serde]
pub struct UpdateEmissionsRequest {
    pub pool: Option<String>,
}

#[cw_serde]
pub struct AccumEmissionsRequest {
    pub address: String,
    pub previous_amount: Uint128,
}

#[cw_serde]
pub struct EndEpochRequest {
    pub limit: Option<u64>,
}

#[cw_serde]
pub struct SetEmissionTokenRequest {
    pub denom: String,
}

#[cw_serde]
pub struct UserVotesRequest {
    pub user: Addr,
    pub epoch: Option<u64>,
}

#[cw_serde]
pub struct TotalGaugeVotesRequest {
    pub gauge: String,
    pub epoch: Option<u64>,
}

#[cw_serde]
pub struct UserGaugeVotesRequest {
    pub user: String,
    pub gauge: String,
    pub epoch: Option<u64>,
}

#[cw_serde]
pub struct GaugeFromPoolRequest {
    pub pool: String,
}

#[cw_serde]
pub struct AllGaugesRequest {
    pub limit: u64,
    pub last_pool: Option<Addr>,
}

#[cw_serde]
pub struct UserVotesResponse {
    pub votes: HashMap<Addr, Uint128>,
    pub epoch: u64,
}

#[cw_serde]
pub struct TotalGaugeVotesResponse {
    pub votes: Uint128,
    pub total_votes: Uint128,
    pub epoch: u64,
}

#[cw_serde]
pub struct UserGaugeVotesResponse {
    pub votes: Uint128,
    pub total_votes: Uint128,
    pub epoch: u64,
}

#[cw_serde]
pub struct GaugeFromPoolResponse {
    pub pool: Addr,
    pub gauge: Addr,
    pub bribes: Vec<Asset>,
    pub fees: Vec<Asset>,
}

#[cw_serde]
pub struct GaugeResponse {
    pub pool: Addr,
    pub gauge: Addr,
}

#[cw_serde]
pub struct AllGaugesResponse {
    pub gauges: Vec<GaugeResponse>,
}

#[cw_serde]
pub struct ConfigResponse {
    /// Timestamp of first epoch
    pub start_time: u64,
    /// Current epoch count, starting from 0
    pub current_epoch: u64,
    pub epoch_duration: u64,
    pub gauge_code_id: u64,
}

#[cw_serde]
pub struct EmissionRewardsRequest {
    pub user: String,
    pub pool_token: String,
}

#[cw_serde]
pub struct EmissionRewardsResponse {
    pub amount: Uint128,
}
