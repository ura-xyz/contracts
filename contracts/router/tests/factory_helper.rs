#![cfg(not(tarpaulin_include))]

use anyhow::Result as AnyResult;
use cosmwasm_std::{Addr, Binary};
use cw20::MinterResponse;
use cw_multi_test::{App, AppResponse, ContractWrapper, Executor};
use ura::contracts::factory::{PairConfig, PairType, QueryMsg};
use ura::structs::asset_info::AssetInfo;
use ura::structs::pair_info::PairInfo;

pub struct FactoryHelper {
    pub owner: Addr,
    pub ura_token: Addr,
    pub factory: Addr,
    pub cw20_token_code_id: u64,
}

impl FactoryHelper {
    pub fn init(router: &mut App, owner: &Addr) -> Self {
        let ura_token_contract = Box::new(ContractWrapper::new_with_empty(
            ura_token::contract::execute,
            ura_token::contract::instantiate,
            ura_token::contract::query,
        ));

        let cw20_token_code_id = router.store_code(ura_token_contract);

        let msg = ura::contracts::token::InstantiateMsg {
            name: String::from("Ura token"),
            symbol: String::from("URA"),
            decimals: 6,
            initial_balances: vec![],
            mint: Some(MinterResponse {
                minter: owner.to_string(),
                cap: None,
            }),
            marketing: None,
        };

        let ura_token = router
            .instantiate_contract(
                cw20_token_code_id,
                owner.clone(),
                &msg,
                &[],
                String::from("URA"),
                None,
            )
            .unwrap();

        let pair_contract = Box::new(
            ContractWrapper::new_with_empty(
                ura_pair::contract::execute,
                ura_pair::contract::instantiate,
                ura_pair::contract::query,
            )
            .with_reply_empty(ura_pair::contract::reply),
        );

        let pair_code_id = router.store_code(pair_contract);

        let factory_contract = Box::new(
            ContractWrapper::new_with_empty(
                ura_factory::executes::execute,
                ura_factory::contract::instantiate,
                ura_factory::queries::query,
            )
            .with_reply_empty(ura_factory::contract::reply),
        );

        let factory_code_id = router.store_code(factory_contract);

        let msg = ura::contracts::factory::InstantiateMsg {
            pair_configs: vec![
                PairConfig {
                    code_id: pair_code_id,
                    pair_type: PairType::Xyk,
                    total_fee_bps: 0,
                    is_disabled: false,
                    is_controller_disabled: false,
                },
                PairConfig {
                    code_id: pair_code_id,
                    pair_type: PairType::Stable,
                    total_fee_bps: 0,
                    is_disabled: false,
                    is_controller_disabled: false,
                },
            ],
            controller_address: None,
            owner: owner.to_string(),
            coin_registry_address: "coin_registry".to_string(),
            token_code_id: cw20_token_code_id,
            fee_address: None,
        };

        let factory = router
            .instantiate_contract(
                factory_code_id,
                owner.clone(),
                &msg,
                &[],
                String::from("URA"),
                None,
            )
            .unwrap();

        Self {
            owner: owner.clone(),
            ura_token,
            factory,
            cw20_token_code_id,
        }
    }

    pub fn create_pair(
        &mut self,
        router: &mut App,
        sender: &Addr,
        pair_type: PairType,
        tokens: [&Addr; 2],
        init_params: Option<Binary>,
    ) -> AnyResult<AppResponse> {
        let asset_infos = vec![
            AssetInfo::Token {
                contract_addr: tokens[0].clone(),
            },
            AssetInfo::Token {
                contract_addr: tokens[1].clone(),
            },
        ];

        let msg = ura::contracts::factory::ExecuteMsg::CreatePair {
            pair_type,
            asset_infos,
            init_params,
            toggle_cw20_token: Some(true),
        };

        router.execute_contract(sender.clone(), self.factory.clone(), &msg, &[])
    }

    pub fn create_pair_with_addr(
        &mut self,
        router: &mut App,
        sender: &Addr,
        pair_type: PairType,
        tokens: [&Addr; 2],
        init_params: Option<Binary>,
    ) -> AnyResult<Addr> {
        self.create_pair(router, sender, pair_type, tokens, init_params)?;

        let asset_infos = vec![
            AssetInfo::Token {
                contract_addr: tokens[0].clone(),
            },
            AssetInfo::Token {
                contract_addr: tokens[1].clone(),
            },
        ];

        let res: PairInfo = router.wrap().query_wasm_smart(
            self.factory.clone(),
            &QueryMsg::Pair {
                asset_infos: asset_infos.clone(),
            },
        )?;

        Ok(res.contract_addr)
    }
}

pub fn instantiate_token(
    app: &mut App,
    token_code_id: u64,
    owner: &Addr,
    token_name: &str,
    decimals: Option<u8>,
) -> Addr {
    let init_msg = ura::contracts::token::InstantiateMsg {
        name: token_name.to_string(),
        symbol: token_name.to_string(),
        decimals: decimals.unwrap_or(6),
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: owner.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    app.instantiate_contract(
        token_code_id,
        owner.clone(),
        &init_msg,
        &[],
        token_name,
        None,
    )
    .unwrap()
}

pub fn mint(
    app: &mut App,
    owner: &Addr,
    token: &Addr,
    amount: u128,
    receiver: &Addr,
) -> AnyResult<AppResponse> {
    app.execute_contract(
        owner.clone(),
        token.clone(),
        &cw20::Cw20ExecuteMsg::Mint {
            recipient: receiver.to_string(),
            amount: amount.into(),
        },
        &[],
    )
}
