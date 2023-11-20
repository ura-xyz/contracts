use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response,
    StdError, StdResult,
};
use cw20::{EmbeddedLogo, Logo, LogoInfo, MarketingInfoResponse};

use cw2::{get_contract_version, set_contract_version};
use cw20_base::contract::{create_accounts, execute as cw20_execute, query as cw20_query};
use cw20_base::msg::{ExecuteMsg as CW20ExecuteMsg, QueryMsg as CW20QueryMsg};
use cw20_base::state::{MinterData, TokenInfo, LOGO, MARKETING_INFO, TOKEN_INFO};
use cw20_base::ContractError;

use ura::contracts::token::{InstantiateMsg, MigrateMsg};
use ura::contracts::xp_token::{ExecuteMsg, OwnerResponse, QueryMsg, WhitelistResponse};
use ura::utils::validation::addr_opt_validate;

use crate::state::{OWNER, WHITELISTED_ADDRESS};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "xp-token";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const LOGO_SIZE_CAP: usize = 5 * 1024;

/// Checks if data starts with XML preamble
fn verify_xml_preamble(data: &[u8]) -> Result<(), ContractError> {
    // The easiest way to perform this check would be just match on regex, however regex
    // compilation is heavy and probably not worth it.

    let preamble = data
        .split_inclusive(|c| *c == b'>')
        .next()
        .ok_or(ContractError::InvalidXmlPreamble {})?;

    const PREFIX: &[u8] = b"<?xml ";
    const POSTFIX: &[u8] = b"?>";

    if !(preamble.starts_with(PREFIX) && preamble.ends_with(POSTFIX)) {
        Err(ContractError::InvalidXmlPreamble {})
    } else {
        Ok(())
    }

    // Additionally attributes format could be validated as they are well defined, as well as
    // comments presence inside of preable, but it is probably not worth it.
}

/// Validates XML logo
fn verify_xml_logo(logo: &[u8]) -> Result<(), ContractError> {
    verify_xml_preamble(logo)?;

    if logo.len() > LOGO_SIZE_CAP {
        Err(ContractError::LogoTooBig {})
    } else {
        Ok(())
    }
}

/// Validates png logo
fn verify_png_logo(logo: &[u8]) -> Result<(), ContractError> {
    // PNG header format:
    // 0x89 - magic byte, out of ASCII table to fail on 7-bit systems
    // "PNG" ascii representation
    // [0x0d, 0x0a] - dos style line ending
    // 0x1a - dos control character, stop displaying rest of the file
    // 0x0a - unix style line ending
    const HEADER: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    if logo.len() > LOGO_SIZE_CAP {
        Err(ContractError::LogoTooBig {})
    } else if !logo.starts_with(&HEADER) {
        Err(ContractError::InvalidPngHeader {})
    } else {
        Ok(())
    }
}

/// Checks if passed logo is correct, and if not, returns an error
fn verify_logo(logo: &Logo) -> Result<(), ContractError> {
    match logo {
        Logo::Embedded(EmbeddedLogo::Svg(logo)) => verify_xml_logo(logo),
        Logo::Embedded(EmbeddedLogo::Png(logo)) => verify_png_logo(logo),
        Logo::Url(_) => Ok(()), // Any reasonable url validation would be regex based, probably not worth it
    }
}

/// Creates a new contract with the specified parameters in the [`InstantiateMsg`].
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // check valid token info
    msg.validate()?;
    // create initial accounts
    let total_supply = create_accounts(&mut deps, &msg.initial_balances)?;

    // Check supply cap
    if let Some(limit) = msg.get_cap() {
        if total_supply > limit {
            return Err(StdError::generic_err("Initial supply greater than cap").into());
        }
    }

    let mint = match msg.mint {
        Some(m) => Some(MinterData {
            minter: deps.api.addr_validate(&m.minter)?,
            cap: m.cap,
        }),
        None => None,
    };

    // Store token info
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply,
        mint,
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    if let Some(marketing) = msg.marketing {
        let logo = if let Some(logo) = marketing.logo {
            verify_logo(&logo)?;
            LOGO.save(deps.storage, &logo)?;

            match logo {
                Logo::Url(url) => Some(LogoInfo::Url(url)),
                Logo::Embedded(_) => Some(LogoInfo::Embedded),
            }
        } else {
            None
        };

        let data = MarketingInfoResponse {
            project: marketing.project,
            description: marketing.description,
            marketing: addr_opt_validate(deps.api, &marketing.marketing)?,
            logo,
        };
        MARKETING_INFO.save(deps.storage, &data)?;
    }

    OWNER.save(deps.storage, &info.sender)?;

    Ok(Response::default())
}

/// Exposes execute functions available in the contract.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let owner = OWNER.load(deps.storage)?;

    if info.sender != owner && !WHITELISTED_ADDRESS.has(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    match msg {
        ExecuteMsg::UpdateOwner { address } => {
            if info.sender != owner {
                return Err(ContractError::Unauthorized {});
            }
            OWNER.save(deps.storage, &deps.api.addr_validate(&address)?)?;
            Ok(Response::default())
        }
        ExecuteMsg::AddWhitelist { address } => {
            if info.sender != owner {
                return Err(ContractError::Unauthorized {});
            }
            WHITELISTED_ADDRESS.save(
                deps.storage,
                &deps.api.addr_validate(&address)?,
                &Empty {},
            )?;
            Ok(Response::default())
        }
        ExecuteMsg::RemoveWhitelist { address } => {
            if info.sender != owner {
                return Err(ContractError::Unauthorized {});
            }
            WHITELISTED_ADDRESS.remove(deps.storage, &deps.api.addr_validate(&address)?);
            Ok(Response::default())
        }
        _ => cw20_execute(deps, env, info, CW20ExecuteMsg::try_from(msg)?),
    }
}

/// Exposes queries available in the contract.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Owner {} => {
            let owner = OWNER.load(deps.storage)?;
            Ok(to_json_binary(&OwnerResponse {
                owner: owner.into_string(),
            })?)
        }
        QueryMsg::Whitelist { address } => {
            let is_whitelisted =
                WHITELISTED_ADDRESS.has(deps.storage, &deps.api.addr_validate(&address)?);
            Ok(to_json_binary(&WhitelistResponse {
                is_whitelisted: is_whitelisted,
            })?)
        }
        _ => cw20_query(deps, env, CW20QueryMsg::try_from(msg)?),
    }
}

/// Manages contract migration.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    let contract_version = get_contract_version(deps.storage)?;

    match contract_version.contract.as_ref() {
        "cw20-token" => match contract_version.version.as_ref() {
            "1.0.0" | "1.1.0" => {}
            _ => {
                return Err(StdError::generic_err(
                    "Cannot migrate. Unsupported contract version",
                ))
            }
        },
        _ => {
            return Err(StdError::generic_err(
                "Cannot migrate. Unsupported contract name",
            ))
        }
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("previous_contract_name", &contract_version.contract)
        .add_attribute("previous_contract_version", &contract_version.version)
        .add_attribute("new_contract_name", CONTRACT_NAME)
        .add_attribute("new_contract_version", CONTRACT_VERSION))
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{Addr, StdError};

    use super::*;
    use ura::contracts::token::InstantiateMarketingInfo;

    mod marketing {
        use cw20::DownloadLogoResponse;
        use cw20_base::contract::{query_download_logo, query_marketing_info};

        use super::*;

        #[test]
        fn basic() {
            let mut deps = mock_dependencies();
            let instantiate_msg = InstantiateMsg {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![],
                mint: None,
                marketing: Some(InstantiateMarketingInfo {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some("marketing".to_owned()),
                    logo: Some(Logo::Url("url".to_owned())),
                }),
            };

            let info = mock_info("creator", &[]);
            let env = mock_env();
            let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
            assert_eq!(0, res.messages.len());

            assert_eq!(
                query_marketing_info(deps.as_ref()).unwrap(),
                MarketingInfoResponse {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some(Addr::unchecked("marketing")),
                    logo: Some(LogoInfo::Url("url".to_owned())),
                }
            );

            let err = query_download_logo(deps.as_ref()).unwrap_err();
            assert!(
                matches!(err, StdError::NotFound { .. }),
                "Expected StdError::NotFound, received {}",
                err
            );
        }

        #[test]
        fn svg() {
            let mut deps = mock_dependencies();
            let img = "<?xml version=\"1.0\"?><svg></svg>".as_bytes();
            let instantiate_msg = InstantiateMsg {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![],
                mint: None,
                marketing: Some(InstantiateMarketingInfo {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some("marketing".to_owned()),
                    logo: Some(Logo::Embedded(EmbeddedLogo::Svg(img.into()))),
                }),
            };

            let info = mock_info("creator", &[]);
            let env = mock_env();
            let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
            assert_eq!(0, res.messages.len());

            assert_eq!(
                query_marketing_info(deps.as_ref()).unwrap(),
                MarketingInfoResponse {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some(Addr::unchecked("marketing")),
                    logo: Some(LogoInfo::Embedded),
                }
            );

            let res: DownloadLogoResponse = query_download_logo(deps.as_ref()).unwrap();
            assert_eq! {
                res,
                DownloadLogoResponse{
                    data: img.into(),
                    mime_type: "image/svg+xml".to_owned(),
                }
            }
        }

        #[test]
        fn png() {
            let mut deps = mock_dependencies();
            const PNG_HEADER: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
            let instantiate_msg = InstantiateMsg {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![],
                mint: None,
                marketing: Some(InstantiateMarketingInfo {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some("marketing".to_owned()),
                    logo: Some(Logo::Embedded(EmbeddedLogo::Png(PNG_HEADER.into()))),
                }),
            };

            let info = mock_info("creator", &[]);
            let env = mock_env();
            let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
            assert_eq!(0, res.messages.len());

            assert_eq!(
                query_marketing_info(deps.as_ref()).unwrap(),
                MarketingInfoResponse {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some(Addr::unchecked("marketing")),
                    logo: Some(LogoInfo::Embedded),
                }
            );

            let res: DownloadLogoResponse = query_download_logo(deps.as_ref()).unwrap();
            assert_eq! {
                res,
                DownloadLogoResponse{
                    data: PNG_HEADER.into(),
                    mime_type: "image/png".to_owned(),
                }
            }
        }
    }
}
