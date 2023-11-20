use cosmwasm_std::{Decimal, StdError, StdResult, Uint128};

pub fn truncate(d: Decimal) -> StdResult<Uint128> {
    let res = (d.atomics() / Uint128::from(1000_000_000_000_000_000u128))
        .try_into()
        .map_err(|_| StdError::generic_err("overflow converting decimal to uint128"))?;
    return Ok(res);
}
