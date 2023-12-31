use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;

use crate::structs::asset_info::AssetInfo;

pub const MAX_SWAP_OPERATIONS: usize = 50;

/// This structure holds the parameters used for creating a contract.
#[cw_serde]
pub struct InstantiateMsg {
    /// The ura factory contract address
    pub ura_factory: String,
}

/// This enum describes a swap operation.
#[cw_serde]
pub struct SwapOperation {
    /// Information about the asset being swapped
    pub offer_asset_info: AssetInfo,
    /// Information about the asset we swap to
    pub ask_asset_info: AssetInfo,
}

/// This structure describes the execute messages available in the contract.
#[cw_serde]
pub enum ExecuteMsg {
    /// Receive receives a message of type [`Cw20ReceiveMsg`] and processes it depending on the received template
    Receive(Cw20ReceiveMsg),
    /// ExecuteSwapOperations processes multiple swaps while mentioning the minimum amount of tokens to receive for the last swap operation
    ExecuteSwapOperations {
        /// A vector of swap operations
        operations: Vec<SwapOperation>,
        /// The recipient
        to: Option<String>,
        /// The minimum amount of tokens to get from a swap
        minimum_receive: Option<Uint128>,
    },

    /// Internal use
    /// ExecuteSwapOperation executes a single swap operation
    ExecuteSwapOperation {
        operation: SwapOperation,
        to: Option<String>,
    },
    /// Internal use
    /// AssertMinimumReceive checks that a receiver will get a minimum amount of tokens from a swap
    AssertMinimumReceive {
        asset_info: AssetInfo,
        prev_balance: Uint128,
        minimum_receive: Uint128,
        receiver: String,
    },
}

#[cw_serde]
pub enum Cw20HookMsg {
    ExecuteSwapOperations {
        /// A vector of swap operations
        operations: Vec<SwapOperation>,
        /// The recipient
        to: Option<String>,
        /// The minimum amount of tokens to get from a swap
        minimum_receive: Option<Uint128>,
    },
}

/// This structure describes the query messages available in the contract.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Config returns configuration parameters for the contract using a custom [`ConfigResponse`] structure
    #[returns(ConfigResponse)]
    Config {},
    /// SimulateSwapOperations simulates multi-hop swap operations
    #[returns(SimulateSwapOperationsResponse)]
    SimulateSwapOperations {
        /// The amount of tokens to swap
        offer_amount: Uint128,
        /// The swap operations to perform, each swap involving a specific pool
        operations: Vec<SwapOperation>,
    },
}

/// This structure describes a custom struct to return a query response containing the base contract configuration.
#[cw_serde]
pub struct ConfigResponse {
    /// The ura factory contract address
    pub ura_factory: String,
}

/// This structure describes a custom struct to return a query response containing the end amount of a swap simulation
#[cw_serde]
pub struct SimulateSwapOperationsResponse {
    /// The amount of tokens received in a swap simulation
    pub amount: Uint128,
}

/// This structure describes a migration message.
/// We currently take no arguments for migrations.
#[cw_serde]
pub struct MigrateMsg {}
