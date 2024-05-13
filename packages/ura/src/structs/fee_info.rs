use cosmwasm_std::{Addr, Decimal};

/// This structure holds parameters that describe the fee structure for a pool.
pub struct FeeInfo {
    /// The controller address
    pub controller_address: Option<Addr>,
    /// The gauge address
    pub gauge_address: Option<Addr>,
    /// The fee address that accumulates the fees in phrase 1
    pub fee_address: Option<Addr>,
    /// The total amount of fees charged per swap
    pub total_fee_rate: Decimal,
}
