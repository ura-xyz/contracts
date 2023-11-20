use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Deps, StdResult, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    /// contract addr of the token to stake
    pub token: String,
    /// contract addr of the es token (minter)
    pub es_minter: String,
    /// Length of an epoch in seconds
    pub epoch_start_time: u64,
    pub epoch_duration: u64,
    pub min_lock: u64,
    pub max_lock: u64,
}

#[cw_serde]
pub struct WithdrawRequest {
    pub id: u64,
}

#[cw_serde]
pub struct UserVotedRequest {
    pub user: String,
}

#[cw_serde]
pub struct ExtendRequest {
    pub id: u64,
    pub epochs_to_extend: u64,
}

#[cw_serde]
pub struct BondRequest {
    pub epochs_to_lock: u64,
    pub for_user: Option<String>,
}

#[cw_serde]
pub struct RebaseRequest {
    pub for_epoch: u64,
}

#[cw_serde]
pub struct UpdateConfigRequest {
    pub es_minter: Option<String>,
    pub controller: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Claim is used to claim your native tokens that you previously "unbonded"
    /// after the contract-defined waiting period
    Withdraw(WithdrawRequest),

    /// Claim rebase rewards
    ClaimRebase(),

    /// Extend an existing stake
    Extend(ExtendRequest),

    /// Record vote
    UserVoted(UserVotedRequest),

    /// Update config
    UpdateConfig(UpdateConfigRequest),

    /// Bond tokens
    Bond(BondRequest),

    /// Rebase
    Rebase(RebaseRequest),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AdminResponse)]
    Admin {},
    #[returns(TotalVeSupplyResponse)]
    TotalVotingPower { at_epoch: Option<u64> },
    #[returns(VotingPowerResponse)]
    IndividualVotingPower { addr: String },
    #[returns(StakedResponse)]
    Staked { addr: String, id: u64 },
    #[returns(AllStakedResponse)]
    AllStakes { addr: String },
    #[returns(ClaimResponse)]
    Claims { addr: String, id: u64 },
    #[returns(RebaseClaimableResponse)]
    RebaseClaimable { addr: String },
}

#[cw_serde]
pub struct MigrateMsg {}

pub fn query_total_voting_power(
    deps: Deps,
    at_epoch: Option<u64>,
    contract_addr: Addr,
) -> StdResult<TotalVeSupplyResponse> {
    deps.querier
        .query_wasm_smart(contract_addr, &QueryMsg::TotalVotingPower { at_epoch })
}

#[cw_serde]
pub struct AdminResponse {
    pub admin: Option<String>,
}

/// A group member has a weight associated with them.
/// This may all be equal, or may have meaning in the app that
/// makes use of the group (eg. voting power)
#[cw_serde]
pub struct VotingPower {
    pub addr: String,
    pub weight: Uint128,
}

#[cw_serde]
pub struct VotingPowerListResponse {
    pub members: Vec<VotingPower>,
}

#[cw_serde]
pub struct VotingPowerResponse {
    pub weight: Uint128,
}

#[cw_serde]
pub struct TotalVeSupplyResponse {
    pub weight: Uint128,
}

#[cw_serde]
pub struct ClaimResponse {
    pub claim: Uint128,
}

#[cw_serde]
pub struct StakedResponse {
    pub stake: Uint128,
}

#[cw_serde]
pub struct Stake {
    pub epochs_left: u64,
    pub amount: Uint128,
    pub weight: Uint128,
    pub rebase: Uint128,
}

#[cw_serde]
pub struct AllStakedResponse {
    pub stakes: Vec<Stake>,
}

#[cw_serde]
pub struct RebaseClaimableResponse {
    pub amount: Uint128,
}
