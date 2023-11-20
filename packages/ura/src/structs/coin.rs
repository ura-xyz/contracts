use super::asset::Asset;
use super::asset_info::AssetInfo;
use cosmwasm_std::{coin, ensure, Coin, StdError, StdResult};
use itertools::Itertools;

pub trait CoinsExt {
    fn assert_coins_properly_sent(
        &self,
        assets: &[Asset],
        pool_asset_infos: &[AssetInfo],
    ) -> StdResult<()>;
}

impl CoinsExt for Vec<Coin> {
    fn assert_coins_properly_sent(
        &self,
        input_assets: &[Asset],
        pool_asset_infos: &[AssetInfo],
    ) -> StdResult<()> {
        ensure!(
            !input_assets.is_empty(),
            StdError::generic_err("Empty input assets")
        );

        ensure!(
            input_assets.iter().map(|asset| &asset.info).all_unique(),
            StdError::generic_err("Duplicated assets in the input")
        );

        input_assets.iter().try_for_each(|input| {
            if pool_asset_infos.contains(&input.info) {
                match &input.info {
                    AssetInfo::NativeToken { denom } => {
                        let coin = self
                            .iter()
                            .find(|coin| coin.denom == *denom)
                            .cloned()
                            .unwrap_or_else(|| coin(0, denom));
                        if coin.amount != input.amount {
                            Err(StdError::generic_err(
                                format!("Native token balance mismatch between the argument and the transferred"),
                            ))
                        } else {
                            Ok(())
                        }
                    }
                    AssetInfo::Token { .. } => Ok(())
                }
            } else {
                Err(StdError::generic_err(format!(
                    "Asset {} is not in the pool",
                    input.info
                )))
            }
        })?;

        self.iter().try_for_each(|coin| {
            if pool_asset_infos.contains(&AssetInfo::NativeToken {
                denom: coin.denom.clone(),
            }) {
                Ok(())
            } else {
                Err(StdError::generic_err(format!(
                    "Transferred coin {} is not in the pool",
                    coin.denom
                )))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;

    use crate::structs::asset_info::{native_asset_info, token_asset_info, AssetInfoExt};

    use super::*;

    #[test]
    fn test_proper_native_coins_sent() {
        let pool_asset_infos = [
            native_asset_info("uusd".to_string()),
            native_asset_info("uluna".to_string()),
        ];

        let assets = [
            pool_asset_infos[0].with_balance(1000u16),
            pool_asset_infos[1].with_balance(1000u16),
        ];
        vec![coin(1000, "uusd"), coin(1000, "uluna")]
            .assert_coins_properly_sent(&assets, &pool_asset_infos)
            .unwrap();

        let assets = [
            pool_asset_infos[0].with_balance(1000u16),
            pool_asset_infos[1].with_balance(100u16),
        ];
        let err = vec![]
            .assert_coins_properly_sent(&assets, &pool_asset_infos)
            .unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err(
                "Native token balance mismatch between the argument and the transferred"
            )
        );

        let assets = [
            pool_asset_infos[0].with_balance(1000u16),
            pool_asset_infos[1].with_balance(1000u16),
        ];
        let err = vec![coin(1000, "uusd"), coin(1000, "random")]
            .assert_coins_properly_sent(&assets, &pool_asset_infos)
            .unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err(
                "Native token balance mismatch between the argument and the transferred"
            )
        );

        let assets = [
            pool_asset_infos[0].with_balance(1000u16),
            native_asset_info("random".to_string()).with_balance(100u16),
        ];
        let err = vec![coin(1000, "uusd"), coin(100, "random")]
            .assert_coins_properly_sent(&assets, &pool_asset_infos)
            .unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err("Asset random is not in the pool")
        );

        let assets = [
            pool_asset_infos[0].with_balance(1000u16),
            pool_asset_infos[1].with_balance(1000u16),
        ];
        let err = vec![coin(1000, "uusd"), coin(100, "uluna")]
            .assert_coins_properly_sent(&assets, &pool_asset_infos)
            .unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err(
                "Native token balance mismatch between the argument and the transferred"
            )
        );

        let pool_asset_infos = [
            token_asset_info(Addr::unchecked("addr0000")),
            token_asset_info(Addr::unchecked("addr0001")),
        ];
        let assets = [
            pool_asset_infos[0].with_balance(1000u16),
            pool_asset_infos[1].with_balance(1000u16),
        ];
        let err = vec![coin(1000, "uusd"), coin(1000, "uluna")]
            .assert_coins_properly_sent(&assets, &pool_asset_infos)
            .unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err("Transferred coin uusd is not in the pool")
        );
    }
}
