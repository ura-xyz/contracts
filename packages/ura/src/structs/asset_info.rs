use std::fmt;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    from_json, Addr, Api, CustomQuery, Decimal256, QuerierWrapper, StdError, StdResult, Uint128,
};
use cw20::Denom;
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};

use crate::utils::querier::{query_balance, query_token_balance, query_token_precision};
use crate::utils::validation::validate_native_denom;

use super::asset::Asset;
use super::decimal256_asset::Decimal256Asset;

/// This enum describes available Token types.
#[cw_serde]
#[derive(Hash, Eq)]
pub enum AssetInfo {
    /// Non-native Token
    Token { contract_addr: Addr },
    /// Native token
    NativeToken { denom: String },
}

impl<'a> PrimaryKey<'a> for &AssetInfo {
    type Prefix = ();

    type SubPrefix = ();

    type Suffix = Self;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        vec![Key::Ref(self.as_bytes())]
    }
}

impl<'a> Prefixer<'a> for &AssetInfo {
    fn prefix(&self) -> Vec<Key> {
        vec![Key::Ref(self.as_bytes())]
    }
}

impl KeyDeserialize for &AssetInfo {
    type Output = AssetInfo;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        from_json(&value)
    }
}

impl fmt::Display for AssetInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AssetInfo::NativeToken { denom } => write!(f, "{denom}"),
            AssetInfo::Token { contract_addr } => write!(f, "{contract_addr}"),
        }
    }
}

impl From<Denom> for AssetInfo {
    fn from(denom: Denom) -> Self {
        match denom {
            Denom::Cw20(contract_addr) => token_asset_info(contract_addr),
            Denom::Native(denom) => native_asset_info(denom),
        }
    }
}

impl From<AssetInfo> for Denom {
    fn from(asset_info: AssetInfo) -> Self {
        match asset_info {
            AssetInfo::Token { contract_addr } => Denom::Cw20(contract_addr),
            AssetInfo::NativeToken { denom } => Denom::Native(denom),
        }
    }
}

impl TryFrom<AssetInfo> for Addr {
    type Error = StdError;

    fn try_from(asset_info: AssetInfo) -> StdResult<Self> {
        match asset_info {
            AssetInfo::Token { contract_addr } => Ok(contract_addr),
            AssetInfo::NativeToken { denom: _ } => Err(StdError::generic_err("Not a CW20 token")),
        }
    }
}

impl From<Addr> for AssetInfo {
    fn from(contract_addr: Addr) -> Self {
        token_asset_info(contract_addr)
    }
}

impl AssetInfo {
    /// Returns an [`AssetInfo`] object representing the denomination for native asset.
    pub fn native<A: Into<String>>(denom: A) -> Self {
        native_asset_info(denom.into())
    }

    /// Returns an [`AssetInfo`] object representing the address of a CW20 token contract.
    pub fn cw20(contract_addr: Addr) -> Self {
        token_asset_info(contract_addr)
    }

    /// Returns an [`AssetInfo`] object representing the address of a CW20 token contract, bypassing
    /// the address validation.
    pub fn cw20_unchecked<A: Into<String>>(contract_addr: A) -> Self {
        AssetInfo::Token {
            contract_addr: Addr::unchecked(contract_addr.into()),
        }
    }

    /// Returns true if the caller is a native token. Otherwise returns false.
    pub fn is_native_token(&self) -> bool {
        match self {
            AssetInfo::NativeToken { .. } => true,
            AssetInfo::Token { .. } => false,
        }
    }

    /// Checks whether the native coin is IBCed token or not.
    pub fn is_ibc(&self) -> bool {
        match self {
            AssetInfo::NativeToken { denom } => denom.to_lowercase().starts_with("ibc/"),
            AssetInfo::Token { .. } => false,
        }
    }

    /// Returns the balance of token in a pool.
    ///
    /// * **pool_addr** is the address of the contract whose token balance we check.
    pub fn query_pool<C>(
        &self,
        querier: &QuerierWrapper<C>,
        pool_addr: impl Into<String>,
    ) -> StdResult<Uint128>
    where
        C: CustomQuery,
    {
        match self {
            AssetInfo::Token { contract_addr, .. } => {
                query_token_balance(querier, contract_addr, pool_addr)
            }
            AssetInfo::NativeToken { denom } => query_balance(querier, pool_addr, denom),
        }
    }

    /// Returns the number of decimals that a token has.
    pub fn decimals<C>(&self, querier: &QuerierWrapper<C>, factory_addr: &Addr) -> StdResult<u8>
    where
        C: CustomQuery,
    {
        query_token_precision(querier, self, factory_addr)
    }

    /// Returns **true** if the calling token is the same as the token specified in the input parameters.
    /// Otherwise returns **false**.
    pub fn equal(&self, asset: &AssetInfo) -> bool {
        match (self, asset) {
            (AssetInfo::NativeToken { denom }, AssetInfo::NativeToken { denom: other_denom }) => {
                denom == other_denom
            }
            (
                AssetInfo::Token { contract_addr },
                AssetInfo::Token {
                    contract_addr: other_contract_addr,
                },
            ) => contract_addr == other_contract_addr,
            _ => false,
        }
    }

    /// If the caller object is a native token of type [`AssetInfo`] then his `denom` field converts to a byte string.
    ///
    /// If the caller object is a token of type [`AssetInfo`] then its `contract_addr` field converts to a byte string.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            AssetInfo::NativeToken { denom } => denom.as_bytes(),
            AssetInfo::Token { contract_addr } => contract_addr.as_bytes(),
        }
    }

    /// Checks that the tokens' denom or contract addr is valid.
    pub fn check(&self, api: &dyn Api) -> StdResult<()> {
        match self {
            AssetInfo::Token { contract_addr } => {
                api.addr_validate(contract_addr.as_str())?;
            }
            AssetInfo::NativeToken { denom } => {
                validate_native_denom(denom)?;
            }
        }

        Ok(())
    }
}

/// Trait extension for AssetInfo to produce [`Asset`] objects from [`AssetInfo`].
pub trait AssetInfoExt {
    fn with_balance(&self, balance: impl Into<Uint128>) -> Asset;
    fn with_dec_balance(&self, balance: Decimal256) -> Decimal256Asset;
}

impl AssetInfoExt for AssetInfo {
    fn with_balance(&self, balance: impl Into<Uint128>) -> Asset {
        Asset {
            info: self.clone(),
            amount: balance.into(),
        }
    }

    fn with_dec_balance(&self, balance: Decimal256) -> Decimal256Asset {
        Decimal256Asset {
            info: self.clone(),
            amount: balance,
        }
    }
}

/// Returns an [`AssetInfo`] object representing the denomination for native asset.
pub fn native_asset_info(denom: String) -> AssetInfo {
    AssetInfo::NativeToken { denom }
}

/// Returns an [`AssetInfo`] object representing the address of a token contract.
pub fn token_asset_info(contract_addr: Addr) -> AssetInfo {
    AssetInfo::Token { contract_addr }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_asset_info() {
        let info = AssetInfo::native("uusd");
        assert_eq!(
            AssetInfo::NativeToken {
                denom: "uusd".to_string()
            },
            info
        );
    }

    #[test]
    fn cw20_unchecked_asset_info() {
        let info = AssetInfo::cw20_unchecked(Addr::unchecked("mock_token"));
        assert_eq!(
            AssetInfo::Token {
                contract_addr: Addr::unchecked("mock_token")
            },
            info
        );
    }

    #[test]
    fn cw20_asset_info() {
        let info = AssetInfo::cw20(Addr::unchecked("mock_token"));
        assert_eq!(
            AssetInfo::Token {
                contract_addr: Addr::unchecked("mock_token")
            },
            info
        );
    }

    #[test]
    fn test_from_addr_for_asset_info() {
        let addr = Addr::unchecked("mock_token");
        let info = AssetInfo::from(addr.clone());
        assert_eq!(info, AssetInfo::cw20(addr));
    }

    #[test]
    fn test_try_from_asset_info_for_addr() {
        let addr = Addr::unchecked("mock_token");
        let info = AssetInfo::cw20(addr.clone());
        let addr2: Addr = info.try_into().unwrap();
        assert_eq!(addr, addr2);
    }

    #[test]
    fn test_from_denom_for_asset_info() {
        let denom = Denom::Native("uusd".to_string());
        let info = AssetInfo::from(denom.clone());
        assert_eq!(info, AssetInfo::native("uusd"));
    }

    #[test]
    fn test_try_from_asset_info_for_denom() {
        let denom = Denom::Native("uusd".to_string());
        let info = AssetInfo::native("uusd");
        let denom2: Denom = info.try_into().unwrap();
        assert_eq!(denom, denom2);
    }
}
