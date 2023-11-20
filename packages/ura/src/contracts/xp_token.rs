use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, StdError, Uint128};
use cw20::{Expiration, Logo};
use cw20_base::msg::{ExecuteMsg as CW20ExecuteMsg, QueryMsg as CW20QueryMsg};

#[cw_serde]
pub enum ExecuteMsg {
    AddWhitelist {
        address: String,
    },
    RemoveWhitelist {
        address: String,
    },
    UpdateOwner {
        address: String,
    },

    /**
     * This includes cw20_base::contract::execute
     */

    /// Transfer is a base message to move tokens to another account without triggering actions
    Transfer {
        recipient: String,
        amount: Uint128,
    },
    /// Burn is a base message to destroy tokens forever
    Burn {
        amount: Uint128,
    },
    /// Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Only with "approval" extension. Allows spender to access an additional amount tokens
    /// from the owner's (env.sender) account. If expires is Some(), overwrites current allowance
    /// expiration with this one.
    IncreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Only with "approval" extension. Lowers the spender's access of tokens
    /// from the owner's (env.sender) account by amount. If expires is Some(), overwrites current
    /// allowance expiration with this one.
    DecreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Only with "approval" extension. Transfers amount tokens from owner -> recipient
    /// if `env.sender` has sufficient pre-approval.
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    /// Only with "approval" extension. Sends amount tokens from owner -> contract
    /// if `env.sender` has sufficient pre-approval.
    SendFrom {
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Only with "approval" extension. Destroys tokens forever
    BurnFrom {
        owner: String,
        amount: Uint128,
    },
    /// Only with the "mintable" extension. If authorized, creates amount new tokens
    /// and adds to the recipient balance.
    Mint {
        recipient: String,
        amount: Uint128,
    },
    /// Only with the "mintable" extension. The current minter may set
    /// a new minter. Setting the minter to None will remove the
    /// token's minter forever.
    UpdateMinter {
        new_minter: Option<String>,
    },
    /// Only with the "marketing" extension. If authorized, updates marketing metadata.
    /// Setting None/null for any of these will leave it unchanged.
    /// Setting Some("") will clear this field on the contract storage
    UpdateMarketing {
        /// A URL pointing to the project behind this token.
        project: Option<String>,
        /// A longer description of the token and it's utility. Designed for tooltips or such
        description: Option<String>,
        /// The address (if any) who can update this data structure
        marketing: Option<String>,
    },
    /// If set as the "marketing" role on the contract, upload a new URL, SVG, or PNG for the token
    UploadLogo(Logo),
}

impl TryFrom<ExecuteMsg> for CW20ExecuteMsg {
    type Error = StdError;

    fn try_from(msg: ExecuteMsg) -> Result<Self, Self::Error> {
        match msg {
            ExecuteMsg::UpdateOwner { .. } => Err(StdError::parse_err(
                "CW20ExecuteMsg",
                "Cannot convert XpExecuteMsg to CW20ExecuteMsg",
            )),
            ExecuteMsg::AddWhitelist { .. } => Err(StdError::parse_err(
                "CW20ExecuteMsg",
                "Cannot convert XpExecuteMsg to CW20ExecuteMsg",
            )),
            ExecuteMsg::RemoveWhitelist { .. } => Err(StdError::parse_err(
                "CW20ExecuteMsg",
                "Cannot convert XpExecuteMsg to CW20ExecuteMsg",
            )),
            ExecuteMsg::Transfer { recipient, amount } => {
                Ok(CW20ExecuteMsg::Transfer { recipient, amount })
            }
            ExecuteMsg::Burn { amount } => Ok(CW20ExecuteMsg::Burn { amount }),
            ExecuteMsg::Send {
                contract,
                amount,
                msg,
            } => Ok(CW20ExecuteMsg::Send {
                contract,
                amount,
                msg,
            }),
            ExecuteMsg::IncreaseAllowance {
                spender,
                amount,
                expires,
            } => Ok(CW20ExecuteMsg::IncreaseAllowance {
                spender,
                amount,
                expires,
            }),
            ExecuteMsg::DecreaseAllowance {
                spender,
                amount,
                expires,
            } => Ok(CW20ExecuteMsg::DecreaseAllowance {
                spender,
                amount,
                expires,
            }),
            ExecuteMsg::TransferFrom {
                owner,
                recipient,
                amount,
            } => Ok(CW20ExecuteMsg::TransferFrom {
                owner,
                recipient,
                amount,
            }),
            ExecuteMsg::SendFrom {
                owner,
                contract,
                amount,
                msg,
            } => Ok(CW20ExecuteMsg::SendFrom {
                owner,
                contract,
                amount,
                msg,
            }),
            ExecuteMsg::BurnFrom { owner, amount } => {
                Ok(CW20ExecuteMsg::BurnFrom { owner, amount })
            }
            ExecuteMsg::Mint { recipient, amount } => {
                Ok(CW20ExecuteMsg::Mint { recipient, amount })
            }
            ExecuteMsg::UpdateMinter { new_minter } => {
                Ok(CW20ExecuteMsg::UpdateMinter { new_minter })
            }
            ExecuteMsg::UpdateMarketing {
                project,
                description,
                marketing,
            } => Ok(CW20ExecuteMsg::UpdateMarketing {
                project,
                description,
                marketing,
            }),
            ExecuteMsg::UploadLogo(logo) => Ok(CW20ExecuteMsg::UploadLogo(logo)),
        }
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OwnerResponse)]
    Owner {},
    #[returns(WhitelistResponse)]
    Whitelist { address: String },

    /**
     * This includes cw20_base::contract::query
     */

    /// Returns the current balance of the given address, 0 if unset.
    #[returns(cw20::BalanceResponse)]
    Balance { address: String },
    /// Returns metadata on the contract - name, decimals, supply, etc.
    #[returns(cw20::TokenInfoResponse)]
    TokenInfo {},
    /// Only with "mintable" extension.
    /// Returns who can mint and the hard cap on maximum tokens after minting.
    #[returns(cw20::MinterResponse)]
    Minter {},
    /// Only with "allowance" extension.
    /// Returns how much spender can use from owner account, 0 if unset.
    #[returns(cw20::AllowanceResponse)]
    Allowance { owner: String, spender: String },
    /// Only with "enumerable" extension (and "allowances")
    /// Returns all allowances this owner has approved. Supports pagination.
    #[returns(cw20::AllAllowancesResponse)]
    AllAllowances {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Only with "enumerable" extension (and "allowances")
    /// Returns all allowances this spender has been granted. Supports pagination.
    #[returns(cw20::AllSpenderAllowancesResponse)]
    AllSpenderAllowances {
        spender: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Only with "enumerable" extension
    /// Returns all accounts that have balances. Supports pagination.
    #[returns(cw20::AllAccountsResponse)]
    AllAccounts {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Only with "marketing" extension
    /// Returns more metadata on the contract to display in the client:
    /// - description, logo, project url, etc.
    #[returns(cw20::MarketingInfoResponse)]
    MarketingInfo {},
    /// Only with "marketing" extension
    /// Downloads the embedded logo data (if stored on chain). Errors if no logo data is stored for this
    /// contract.
    #[returns(cw20::DownloadLogoResponse)]
    DownloadLogo {},
}

#[cw_serde]
pub struct OwnerResponse {
    pub owner: String,
}

#[cw_serde]
pub struct WhitelistResponse {
    pub is_whitelisted: bool,
}

impl TryFrom<QueryMsg> for CW20QueryMsg {
    type Error = StdError;

    fn try_from(msg: QueryMsg) -> Result<Self, Self::Error> {
        match msg {
            QueryMsg::Owner {} => Err(StdError::parse_err(
                "CW20QueryMsg",
                "Cannot convert XpQueryMsg to CW20QueryMsg",
            )),
            QueryMsg::Whitelist { .. } => Err(StdError::parse_err(
                "CW20QueryMsg",
                "Cannot convert XpQueryMsg to CW20QueryMsg",
            )),
            QueryMsg::Balance { address } => Ok(CW20QueryMsg::Balance { address }),
            QueryMsg::TokenInfo {} => Ok(CW20QueryMsg::TokenInfo {}),
            QueryMsg::Minter {} => Ok(CW20QueryMsg::Minter {}),
            QueryMsg::Allowance { owner, spender } => {
                Ok(CW20QueryMsg::Allowance { owner, spender })
            }
            QueryMsg::AllAllowances {
                owner,
                start_after,
                limit,
            } => Ok(CW20QueryMsg::AllAllowances {
                owner,
                start_after,
                limit,
            }),
            QueryMsg::AllSpenderAllowances {
                spender,
                start_after,
                limit,
            } => Ok(CW20QueryMsg::AllSpenderAllowances {
                spender,
                start_after,
                limit,
            }),
            QueryMsg::AllAccounts { start_after, limit } => {
                Ok(CW20QueryMsg::AllAccounts { start_after, limit })
            }
            QueryMsg::MarketingInfo {} => Ok(CW20QueryMsg::MarketingInfo {}),
            QueryMsg::DownloadLogo {} => Ok(CW20QueryMsg::DownloadLogo {}),
        }
    }
}
