use std::fmt;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    coin, to_json_binary, Addr, BankMsg, Coin, CosmosMsg, CustomMsg, Decimal256, MessageInfo,
    StdError, StdResult, Uint128, WasmMsg,
};
use cw20::{Cw20Coin, Cw20CoinVerified, Cw20ExecuteMsg};
use cw_utils::must_pay;

use super::asset_info::AssetInfo;
use super::decimal256::Decimal256Ext;
use super::decimal256_asset::Decimal256Asset;

#[cw_serde]
pub struct Asset {
    /// Information about an asset stored in a [`AssetInfo`] struct
    pub info: AssetInfo,
    /// A token amount
    pub amount: Uint128,
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.amount, self.info)
    }
}

impl From<Coin> for Asset {
    fn from(coin: Coin) -> Self {
        Asset::native(coin.denom, coin.amount)
    }
}

impl From<&Coin> for Asset {
    fn from(coin: &Coin) -> Self {
        coin.clone().into()
    }
}

impl TryFrom<Asset> for Coin {
    type Error = StdError;

    fn try_from(asset: Asset) -> Result<Self, Self::Error> {
        match asset.info {
            AssetInfo::NativeToken { denom } => Ok(Self {
                denom,
                amount: asset.amount,
            }),
            _ => Err(StdError::parse_err(
                "Asset",
                "Cannot convert non-native asset to Coin",
            )),
        }
    }
}

impl TryFrom<&Asset> for Coin {
    type Error = StdError;

    fn try_from(asset: &Asset) -> Result<Self, Self::Error> {
        asset.clone().try_into()
    }
}

impl From<Cw20CoinVerified> for Asset {
    fn from(coin: Cw20CoinVerified) -> Self {
        Asset::cw20(coin.address, coin.amount)
    }
}

impl TryFrom<Asset> for Cw20CoinVerified {
    type Error = StdError;

    fn try_from(asset: Asset) -> Result<Self, Self::Error> {
        match asset.info {
            AssetInfo::Token { contract_addr } => Ok(Self {
                address: contract_addr,
                amount: asset.amount,
            }),
            _ => Err(StdError::generic_err(
                "Cannot convert non-CW20 asset to Cw20Coin",
            )),
        }
    }
}

impl TryFrom<Asset> for Cw20Coin {
    type Error = StdError;

    fn try_from(asset: Asset) -> Result<Self, Self::Error> {
        let verified: Cw20CoinVerified = asset.try_into()?;
        Ok(Self {
            address: verified.address.to_string(),
            amount: verified.amount,
        })
    }
}

impl Asset {
    /// Constructs a new [`Asset`] object.
    pub fn new<A: Into<Uint128>>(info: AssetInfo, amount: A) -> Self {
        Self {
            info,
            amount: amount.into(),
        }
    }

    /// Returns an [`Asset`] object representing a native token with a given amount.
    pub fn native<A: Into<String>, B: Into<Uint128>>(denom: A, amount: B) -> Self {
        native_asset(denom.into(), amount.into())
    }

    /// Returns an [`Asset`] object representing a CW20 token with a given amount.
    pub fn cw20<A: Into<Uint128>>(contract_addr: Addr, amount: A) -> Self {
        token_asset(contract_addr, amount.into())
    }

    /// Returns an [`Asset`] object representing a CW20 token with a given amount, bypassing the
    /// address validation.
    pub fn cw20_unchecked<A: Into<String>, B: Into<Uint128>>(contract_addr: A, amount: B) -> Self {
        token_asset(Addr::unchecked(contract_addr.into()), amount.into())
    }

    /// Returns true if the token is native. Otherwise returns false.
    pub fn is_native_token(&self) -> bool {
        self.info.is_native_token()
    }

    /// For native tokens of type [`AssetInfo`] uses the default method [`BankMsg::Send`] to send a
    /// token amount to a recipient.
    /// For a token of type [`AssetInfo`] we use the default method [`Cw20ExecuteMsg::Transfer`].
    pub fn into_msg<T>(self, recipient: impl Into<String>) -> StdResult<CosmosMsg<T>>
    where
        T: CustomMsg,
    {
        let recipient = recipient.into();
        match &self.info {
            AssetInfo::Token { contract_addr } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                    recipient,
                    amount: self.amount,
                })?,
                funds: vec![],
            })),
            AssetInfo::NativeToken { .. } => Ok(CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient,
                amount: vec![self.as_coin()?],
            })),
        }
    }

    /// Validates an amount of native tokens being sent.
    pub fn assert_sent_native_token_balance(&self, message_info: &MessageInfo) -> StdResult<()> {
        if let AssetInfo::NativeToken { denom } = &self.info {
            let amount = must_pay(message_info, denom)
                .map_err(|err| StdError::generic_err(err.to_string()))?;
            if self.amount == amount {
                Ok(())
            } else {
                Err(StdError::generic_err(
                    "Native token balance mismatch between the argument and the transferred",
                ))
            }
        } else {
            Ok(())
        }
    }

    pub fn to_decimal_asset(&self, precision: impl Into<u32>) -> StdResult<Decimal256Asset> {
        Ok(Decimal256Asset {
            info: self.info.clone(),
            amount: Decimal256::with_precision(self.amount, precision.into())?,
        })
    }

    pub fn as_coin(&self) -> StdResult<Coin> {
        match &self.info {
            AssetInfo::Token { .. } => {
                Err(StdError::generic_err("Cannot convert token asset to coin"))
            }
            AssetInfo::NativeToken { denom } => Ok(coin(self.amount.u128(), denom)),
        }
    }
}

/// Returns an [`Asset`] object representing a native token and an amount of tokens.
///
/// * **denom** native asset denomination.
///
/// * **amount** amount of native assets.
pub fn native_asset(denom: String, amount: Uint128) -> Asset {
    Asset {
        info: AssetInfo::NativeToken { denom },
        amount,
    }
}

/// Returns an [`Asset`] object representing a non-native token and an amount of tokens.
/// ## Params
/// * **contract_addr** iaddress of the token contract.
///
/// * **amount** amount of tokens.
pub fn token_asset(contract_addr: Addr, amount: Uint128) -> Asset {
    Asset {
        info: AssetInfo::Token { contract_addr },
        amount,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coin;
    use test_case::test_case;

    fn mock_cw20() -> Asset {
        Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked("mock_token"),
            },
            amount: Uint128::new(123456u128),
        }
    }

    fn mock_native() -> Asset {
        Asset {
            info: AssetInfo::NativeToken {
                denom: String::from("uusd"),
            },
            amount: Uint128::new(123456u128),
        }
    }

    #[test]
    fn from_cw20coinverified_for_asset() {
        let coin = Cw20CoinVerified {
            address: Addr::unchecked("mock_token"),
            amount: Uint128::new(123456u128),
        };
        assert_eq!(mock_cw20(), Asset::from(coin));
    }

    #[test]
    fn test_from_coin_for_asset() {
        let coin = coin(123456u128, "uusd");
        assert_eq!(mock_native(), Asset::from(coin));
    }

    #[test_case(mock_native() => matches Err(_) ; "native")]
    #[test_case(mock_cw20() => Ok(Cw20CoinVerified {
                    address: Addr::unchecked("mock_token"),
                    amount: 123456u128.into()
                }) ; "cw20")]
    fn try_from_asset_for_cw20coinverified(asset: Asset) -> StdResult<Cw20CoinVerified> {
        Cw20CoinVerified::try_from(asset)
    }

    #[test_case(mock_native() => matches Err(_) ; "native")]
    #[test_case(mock_cw20() => Ok(Cw20Coin {
                    address: "mock_token".to_string(),
                    amount: 123456u128.into()
                }) ; "cw20")]
    fn try_from_asset_for_cw20coin(asset: Asset) -> StdResult<Cw20Coin> {
        Cw20Coin::try_from(asset)
    }

    #[test]
    fn test_try_from_asset_for_coin() {
        let coin = coin(123456u128, "uusd");
        let asset = Asset::from(&coin);
        let coin2: Coin = asset.try_into().unwrap();
        assert_eq!(coin, coin2);
    }
}
