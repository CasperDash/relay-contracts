#![no_std]
#![no_main]

mod constants;
mod utils;
#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;

use casper_contract::contract_api::{runtime, storage};
use casper_types::account::AccountHash;
use casper_types::contracts::NamedKeys;
use casper_types::{
    CLType, ContractPackageHash, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints,
    Parameter,
};

#[no_mangle]
pub extern "C" fn set_message() {
    let caller: AccountHash = utils::get_caller();
    let message: String = runtime::get_named_arg(constants::ARG_MESSAGE);
    utils::write_storage(constants::KEY_MESSAGE, message);
    utils::write_storage(constants::KEY_CALLER, caller.to_string());
}

#[no_mangle]
pub extern "C" fn call() {
    let relay_contract_package: ContractPackageHash =
        runtime::get_named_arg(constants::KEY_RELAY_CONTRACT_PACKAGE);
    let mut named_keys = NamedKeys::new();
    named_keys.insert(
        constants::KEY_MESSAGE.to_string(),
        storage::new_uref(String::new()).into(),
    );
    named_keys.insert(
        constants::KEY_RELAY_CONTRACT_PACKAGE.to_string(),
        storage::new_uref(relay_contract_package).into(),
    );
    named_keys.insert(
        constants::KEY_CALLER.to_string(),
        storage::new_uref(String::new()).into(),
    );
    // Create the entry points for this contract.
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(EntryPoint::new(
        constants::ENTRY_POINT_SET_MESSAGE,
        vec![Parameter::new(constants::ARG_MESSAGE, CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    // Create a new contract package
    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(constants::CONTRACT_PACKAGE_NAME.to_string()),
        None,
    );

    // Store the contract version in the context's named keys.
    runtime::put_key(
        constants::CONTRACT_VERSION_KEY,
        storage::new_uref(contract_version).into(),
    );
    // Create a named key for the contract hash.
    runtime::put_key(constants::CONTRACT_KEY, contract_hash.into());
}
