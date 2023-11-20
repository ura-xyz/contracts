use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use cw20::Cw20ReceiveMsg;

use crate::structs::asset::Asset;

///
/// Definitions used for a gauge
///

#[cw_serde]
pub struct InstantiateMsg {
    pub pool: String,
    pub epoch: u64,
}

#[cw_serde]
pub struct WithdrawRewardsRequest {
    pub for_user: String,
}

#[cw_serde]
pub struct UserVotedRequest {
    pub user: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Deposit bribes for this gauge. Bribes are distributed at the end of the epoch based on voting weight.
    /// It does not matter when the votes are added. Calculation will be done at the end
    DepositBribe {},
    /// Claims trading fees from the underlying pool. Fees are claimed and distributed to voters who have
    /// already voted. Previously accumulated fees are not distributed to new voters.
    /// Anyone can call this to update the fees claimed but this must be called right before a vote so that
    /// fees can be distributed before the new votes come in
    DepositFees {},
    WithdrawRewards(WithdrawRewardsRequest),
    EndEpoch {},
    UserVoted(UserVotedRequest),
    Receive(Cw20ReceiveMsg),
}

#[cw_serde]
pub enum Cw20HookMsg {
    DepositBribe {},
    DepositFees {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(UserRewardsResponse)]
    UserRewards(WithdrawRewardsRequest),
    #[returns(TotalRewardsResponse)]
    TotalRewards {},
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct UserRewardsResponse {
    pub fees: Vec<Asset>,
    pub bribes: Vec<Asset>,
}

#[cw_serde]
pub struct TotalRewardsResponse {
    pub fees: Vec<Asset>,
    pub bribes: Vec<Asset>,
}

#[cw_serde]
pub struct ConfigResponse {
    /// Supported pool
    pub pool: Addr,
    /// Controller address
    pub controller: Addr,
    /// Epoch number of the current round
    pub epoch: u64,
}
