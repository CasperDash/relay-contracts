#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use casper_contract::contract_api::{account, runtime, system};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::account::AccountHash;
use casper_types::{runtime_args, ContractHash, RuntimeArgs, URef};

const ENTRY_POINT_GET_PURSE: &str = "get_purse";
const ENTRY_POINT_DEPOSIT: &str = "deposit";
const ARG_RELAY_CONTRACT: &str = "relay_contract";
const ARG_AMOUNT: &str = "amount";
const ARG_OWNER: &str = "owner";

#[no_mangle]
pub extern "C" fn call() {
    let relay_contract: ContractHash = runtime::get_named_arg(ARG_RELAY_CONTRACT);
    let amount = runtime::get_named_arg(ARG_AMOUNT);
    let owner: AccountHash = runtime::get_named_arg(ARG_OWNER);
    let purse: URef =
        runtime::call_contract(relay_contract, ENTRY_POINT_GET_PURSE, RuntimeArgs::new());
    system::transfer_from_purse_to_purse(account::get_main_purse(), purse, amount, None)
        .unwrap_or_revert();

    let _: () = runtime::call_contract(
        relay_contract,
        ENTRY_POINT_DEPOSIT,
        runtime_args! {
            ARG_OWNER => owner,
        },
    );
}
