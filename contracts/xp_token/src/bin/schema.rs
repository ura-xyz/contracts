use cosmwasm_schema::write_api;

use cw20_base::msg::{ExecuteMsg, QueryMsg};
use ura::contracts::token::InstantiateMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    }
}
