use cosmwasm_std::{coins, Addr, Api, MessageInfo, StdError, StdResult, Uint128};

use crate::structs::{
    asset::Asset,
    asset_info::{native_asset_info, AssetInfo, AssetInfoExt},
};

/// Minimum initial LP share
pub const MINIMUM_LIQUIDITY_AMOUNT: Uint128 = Uint128::new(1_000);
/// Maximum denom length
pub const DENOM_MAX_LENGTH: usize = 128;

/// Taken from https://github.com/mars-protocol/red-bank/blob/5bb0fe145588352b281803f7b870103bc6832621/packages/utils/src/helpers.rs#L68
/// Follows cosmos SDK validation logic where denom can be 3 - 128 characters long
/// and starts with a letter, followed but either a letter, number, or separator ( ‘/' , ‘:' , ‘.’ , ‘_’ , or '-')
/// reference: https://github.com/cosmos/cosmos-sdk/blob/7728516abfab950dc7a9120caad4870f1f962df5/types/coin.go#L865-L867
pub fn validate_native_denom(denom: &str) -> StdResult<()> {
    if denom.len() < 3 || denom.len() > DENOM_MAX_LENGTH {
        return Err(StdError::generic_err(format!(
            "Invalid denom length [3,{DENOM_MAX_LENGTH}]: {denom}"
        )));
    }

    let mut chars = denom.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() {
        return Err(StdError::generic_err(format!(
            "First character is not ASCII alphabetic: {denom}"
        )));
    }

    let set = ['/', ':', '.', '_', '-'];
    for c in chars {
        if !(c.is_ascii_alphanumeric() || set.contains(&c)) {
            return Err(StdError::generic_err(format!(
                "Not all characters are ASCII alphanumeric or one of:  /  :  .  _  -: {denom}"
            )));
        }
    }

    Ok(())
}

/// Returns a lowercased, validated address upon success if present.
#[inline]
pub fn addr_opt_validate(api: &dyn Api, addr: &Option<String>) -> StdResult<Option<Addr>> {
    addr.as_ref()
        .map(|addr| api.addr_validate(addr))
        .transpose()
}

/// Checks swap parameters.
///
/// * **pools** amount of tokens in pools.
///
/// * **swap_amount** amount to swap.
pub fn check_swap_parameters(pools: Vec<Uint128>, swap_amount: Uint128) -> StdResult<()> {
    if pools.iter().any(|pool| pool.is_zero()) {
        return Err(StdError::generic_err("One of the pools is empty"));
    }

    if swap_amount.is_zero() {
        return Err(StdError::generic_err("Swap amount must not be zero"));
    }

    Ok(())
}

pub fn assert_sent_native_token_balance(
    asset: &Asset,
    message_info: &MessageInfo,
) -> StdResult<()> {
    if let AssetInfo::NativeToken { denom } = &asset.info {
        match message_info.funds.iter().find(|x| x.denom == *denom) {
            Some(coin) => {
                if asset.amount == coin.amount {
                    Ok(())
                } else {
                    Err(StdError::generic_err(
                        "Native token balance mismatch between the argument and the transferred",
                    ))
                }
            }
            None => {
                if asset.amount.is_zero() {
                    Ok(())
                } else {
                    Err(StdError::generic_err(
                        "Native token balance mismatch between the argument and the transferred",
                    ))
                }
            }
        }
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_info;

    #[test]
    fn test_native_coins_sent() {
        let asset = native_asset_info("uusd".to_string()).with_balance(1000u16);

        let info = mock_info("addr0000", &coins(1000, "random"));
        let err = asset.assert_sent_native_token_balance(&info).unwrap_err();
        assert_eq!(err, StdError::generic_err("Must send reserve token 'uusd'"));

        let info = mock_info("addr0000", &coins(100, "uusd"));
        let err = asset.assert_sent_native_token_balance(&info).unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err(
                "Native token balance mismatch between the argument and the transferred"
            )
        );
        let info = mock_info("addr0000", &coins(1000, "uusd"));
        asset.assert_sent_native_token_balance(&info).unwrap();
    }

    #[test]
    fn native_denom_validation() {
        let err = validate_native_denom("ab").unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err("Invalid denom length [3,128]: ab")
        );
        let err = validate_native_denom("1usd").unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err("First character is not ASCII alphabetic: 1usd")
        );
        let err = validate_native_denom("wow@usd").unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err(
                "Not all characters are ASCII alphanumeric or one of:  /  :  .  _  -: wow@usd"
            )
        );
        let long_denom: String = ['a'].repeat(129).iter().collect();
        let err = validate_native_denom(&long_denom).unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err("Invalid denom length [3,128]: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );

        validate_native_denom("uusd").unwrap();
        validate_native_denom(
            "ibc/EBD5A24C554198EBAF44979C5B4D2C2D312E6EBAB71962C92F735499C7575839",
        )
        .unwrap();
        validate_native_denom("factory/wasm1jdppe6fnj2q7hjsepty5crxtrryzhuqsjrj95y/uusd").unwrap();
    }
}
