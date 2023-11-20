use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::{Addr, Binary, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;

use crate::structs::{asset::Asset, asset_info::AssetInfo, pair_info::PairInfo};

/// The default swap slippage (0.005 = 0.5%)
pub const DEFAULT_SLIPPAGE: Decimal = Decimal::raw(5000000000000000u128);
/// The maximum allowed swap slippage (1.0 = 100%)
pub const MAX_ALLOWED_SLIPPAGE: Decimal = Decimal::one();
/// Decimal precision for TWAP results
pub const TWAP_PRECISION: u8 = 6;
/// Minimum initial LP share
pub const MINIMUM_LIQUIDITY_AMOUNT: Uint128 = Uint128::new(1_000);

/// This structure describes the parameters used for creating a contract.
#[cw_serde]
pub struct InstantiateMsg {
    /// Information about assets in the pool
    pub asset_infos: Vec<AssetInfo>,
    /// The factory contract address
    pub factory_addr: String,
    /// Optional binary serialised parameters for custom pool types
    pub init_params: Option<Binary>,
    /// Use cw20 LP token
    pub token_code_id: Option<u64>,
}

/// This structure describes the execute messages available in the contract.
#[cw_serde]
pub enum ExecuteMsg {
    /// Receives a message of type [`Cw20ReceiveMsg`]
    Receive(Cw20ReceiveMsg),
    /// ProvideLiquidity allows someone to provide liquidity in the pool
    ProvideLiquidity {
        /// The assets available in the pool
        assets: Vec<Asset>,
        /// The slippage tolerance that allows liquidity provision only if the price in the pool doesn't move too much
        slippage_tolerance: Option<Decimal>,
        /// The receiver of LP tokens
        receiver: Option<String>,
    },
    /// Swap performs a swap in the pool
    Swap {
        offer_asset: Asset,
        ask_asset_info: Option<AssetInfo>,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
        to: Option<String>,
    },
    /// Update the pair configuration
    UpdateConfig { params: Binary },
    /// Withdraw liquidity from the pool
    WithdrawLiquidity {},
}

/// This structure describes a CW20 hook message.
#[cw_serde]
pub enum Cw20HookMsg {
    /// Swap a given amount of asset
    Swap {
        ask_asset_info: Option<AssetInfo>,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
        to: Option<String>,
    },
    /// Withdraw liquidity from the cw20 LP pool
    WithdrawLiquidity {},
}

/// This structure describes the query messages available in the contract.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns information about a pair in an object of type [`super::asset::PairInfo`].
    #[returns(PairInfo)]
    Pair {},
    /// Returns information about a pool in an object of type [`PoolResponse`].
    #[returns(PoolResponse)]
    Pool {},
    /// Returns contract configuration settings in a custom [`ConfigResponse`] structure.
    #[returns(ConfigResponse)]
    Config {},
    /// Returns information about the share of the pool in a vector that contains objects of type [`Asset`].
    #[returns(Vec<Asset>)]
    Share { amount: Uint128 },
    /// Returns information about a swap simulation in a [`SimulationResponse`] object.
    #[returns(SimulationResponse)]
    Simulation {
        offer_asset: Asset,
        ask_asset_info: Option<AssetInfo>,
    },
    /// Returns information about cumulative prices in a [`ReverseSimulationResponse`] object.
    #[returns(ReverseSimulationResponse)]
    ReverseSimulation {
        offer_asset_info: Option<AssetInfo>,
        ask_asset: Asset,
    },
    /// Returns current D invariant in as a [`u128`] value
    #[returns(Uint128)]
    QueryComputeD {},
    /// Returns the balance of lp tokens received by the address after providing lp
    #[returns(LpReceivedResponse)]
    LpReceived { address: String },
}

/// This struct is used to return a query result with the total amount of LP tokens and assets in a specific pool.
#[cw_serde]
pub struct PoolResponse {
    /// The assets in the pool together with asset amounts
    pub assets: Vec<Asset>,
    /// The total amount of LP tokens currently issued
    pub total_share: Uint128,
}

/// This struct is used to return a query result with the general contract configuration.
#[cw_serde]
pub struct ConfigResponse {
    /// The pool's parameters
    pub params: Option<Binary>,
    /// The contract owner
    pub owner: Addr,
    /// The factory contract address
    pub factory_addr: Addr,
}

/// This structure holds the parameters that are returned from a swap simulation response
#[cw_serde]
pub struct SimulationResponse {
    /// The amount of ask assets returned by the swap
    pub return_amount: Uint128,
    /// The spread used in the swap operation
    pub spread_amount: Uint128,
    /// The amount of fees charged by the transaction
    pub commission_amount: Uint128,
}

/// This structure holds the parameters that are returned from a reverse swap simulation response.
#[cw_serde]
pub struct ReverseSimulationResponse {
    /// The amount of offer assets returned by the reverse swap
    pub offer_amount: Uint128,
    /// The spread used in the swap operation
    pub spread_amount: Uint128,
    /// The amount of fees charged by the transaction
    pub commission_amount: Uint128,
}

/// This structure is used to return a cumulative prices query response.
#[cw_serde]
pub struct CumulativePricesResponse {
    /// The assets in the pool to query
    pub assets: Vec<Asset>,
    /// The total amount of LP tokens currently issued
    pub total_share: Uint128,
    /// The vector contains cumulative prices for each pair of assets in the pool
    pub cumulative_prices: Vec<(AssetInfo, AssetInfo, Uint128)>,
}

/// This structure holds the parameters that are returned from a lp provided response.
#[cw_serde]
pub struct LpReceivedResponse {
    /// The amount of lp tokens that an address received after providing lp
    pub amount: Uint128,
}

/// This structure describes a migration message.
/// We currently take no arguments for migrations.
#[cw_serde]
pub struct MigrateMsg {}

/// This structure holds stableswap pool parameters.
#[cw_serde]
pub struct StablePoolParams {
    /// The current stableswap pool amplification
    pub amp: u64,
    /// The contract owner
    pub owner: Option<String>,
}

/// This structure stores a stableswap pool's configuration.
#[cw_serde]
pub struct StablePoolConfig {
    /// The stableswap pool amplification
    pub amp: Decimal,
}

/// This enum stores the options available to start and stop changing a stableswap pool's amplification.
#[cw_serde]
pub enum StablePoolUpdateParams {
    StartChangingAmp { next_amp: u64, next_amp_time: u64 },
    StopChangingAmp {},
}
