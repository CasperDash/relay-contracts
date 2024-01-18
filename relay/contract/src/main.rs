#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

mod constants;
mod errors;
mod permission;
mod utils;

use crate::errors::Error;
use crate::permission::Permission;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use casper_contract::contract_api::system;
use casper_contract::contract_api::{runtime, storage};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::account::AccountHash;
use casper_types::contracts::NamedKeys;
use casper_types::{
    runtime_args, ApiError, CLType, CLTyped, CLValue, ContractHash, EntryPoint, EntryPointAccess,
    EntryPointType, EntryPoints, Parameter, RuntimeArgs, URef, U512,
};

#[no_mangle]
pub extern "C" fn call() {
    match runtime::get_key(constants::CONTRACT_ACCESS_UREF) {
        None => {
            install_contract();
        }
        Some(_contract_key) => {
            upgrade_contract();
        }
    }
}

#[no_mangle]
pub extern "C" fn init() {
    let installer = utils::get_storage::<AccountHash>(constants::KEY_INSTALLER);
    let caller = runtime::get_caller();
    if caller != installer {
        runtime::revert(ApiError::from(Error::Unauthorized))
    }
    _ = storage::new_dictionary(constants::KEY_REGISTERED_CONTRACT);
    _ = storage::new_dictionary(constants::KEY_OWNER_BALANCE);
    runtime::put_key(constants::KEY_PURSE, system::create_purse().into());
    runtime::put_key(constants::KEY_DEPOSIT_PURSE, system::create_purse().into());
    runtime::put_key(constants::KEY_FEE_PURSE, system::create_purse().into());
}

#[no_mangle]
pub extern "C" fn call_on_behalf() {
    permission::require(Permission::Installer);
    let paymaster = runtime::get_caller();

    let contract_hash: ContractHash = runtime::get_named_arg(constants::ARG_CONTRACT);
    let entry_point: String = runtime::get_named_arg(constants::ARG_ENTRY_POINT);
    let caller: AccountHash = runtime::get_named_arg(constants::ARG_CALLER);
    let gas_amount: U512 = runtime::get_named_arg(constants::ARG_GAS_AMOUNT);
    let pay_amount: U512 = runtime::get_named_arg(constants::ARG_PAY_AMOUNT);

    // Check if recipient contract is registered
    let owner = utils::get_storage_dic::<AccountHash>(
        utils::get_uref(constants::KEY_REGISTERED_CONTRACT),
        contract_hash.to_string().as_str(),
    )
    .unwrap_or_revert_with(ApiError::from(Error::Unregistered));

    let owner_balance = utils::get_storage_dic::<U512>(
        utils::get_uref(constants::KEY_OWNER_BALANCE),
        owner.to_string().as_str(),
    )
    .unwrap();
    let fee_rate = utils::get_storage::<u32>(constants::KEY_FEE_RATE);
    let fee = gas_amount
        .checked_mul(U512::from(fee_rate))
        .unwrap_or_revert()
        .checked_div(U512::from(1000))
        .unwrap_or_revert();
    if owner_balance < gas_amount + fee {
        runtime::revert(ApiError::from(Error::InsufficientBalance))
    }

    let _ = system::transfer_from_purse_to_account(
        utils::get_uref(constants::KEY_DEPOSIT_PURSE),
        paymaster,
        gas_amount,
        None,
    );
    if fee > U512::zero() {
        let _ = system::transfer_from_purse_to_purse(
            utils::get_uref(constants::KEY_DEPOSIT_PURSE),
            utils::get_uref(constants::KEY_FEE_PURSE),
            fee,
            None,
        );
    }

    utils::write_storage_dic(
        utils::get_uref(constants::KEY_OWNER_BALANCE),
        owner.to_string().as_str(),
        owner_balance - gas_amount - fee,
    );

    if pay_amount > U512::zero() {
        let purse = utils::get_uref(constants::KEY_PURSE);
        let recipient_purse: URef = runtime::call_contract(
            contract_hash,
            constants::ENTRY_POINT_GET_PURSE,
            runtime_args! {},
        );
        let _ = system::transfer_from_purse_to_purse(purse, recipient_purse, pay_amount, None);
    }

    let mut args: RuntimeArgs = runtime::get_named_arg(constants::ARG_ARGS);
    args.insert(constants::ARG_CALLER, caller)
        .unwrap_or_revert_with(ApiError::InvalidArgument);

    let _: () = runtime::call_contract(contract_hash, entry_point.as_str(), args);
}

#[no_mangle]
pub extern "C" fn register() {
    permission::require(Permission::Installer);

    let owner: AccountHash = runtime::get_named_arg(constants::ARG_OWNER);
    let owner_balance = utils::get_storage_dic::<U512>(
        utils::get_uref(constants::KEY_OWNER_BALANCE),
        owner.to_string().as_str(),
    );
    if owner_balance.is_none() {
        utils::write_storage_dic(
            utils::get_uref(constants::KEY_OWNER_BALANCE),
            owner.to_string().as_str(),
            U512::zero(),
        );
    }
    let contract_hash: ContractHash = runtime::get_named_arg(constants::ARG_CONTRACT);
    utils::write_storage_dic(
        utils::get_uref(constants::KEY_REGISTERED_CONTRACT),
        contract_hash.to_string().as_str(),
        owner,
    );
}

#[no_mangle]
pub extern "C" fn get_purse() {
    runtime::ret(
        CLValue::from_t(utils::get_uref(constants::KEY_PURSE).into_add()).unwrap_or_revert(),
    );
}

#[no_mangle]
pub extern "C" fn set_fee_rate() {
    permission::require(Permission::Installer);

    let fee_rate: u32 = runtime::get_named_arg(constants::ARG_FEE_RATE);
    utils::write_storage(constants::KEY_FEE_RATE, fee_rate)
}

#[no_mangle]
pub extern "C" fn claim_fee() {
    permission::require(Permission::Installer);
    let caller = runtime::get_caller();

    let fee_purse = runtime::get_key(constants::KEY_FEE_PURSE)
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert();
    let fee_purse_balance = system::get_purse_balance(fee_purse).unwrap_or_revert();

    system::transfer_from_purse_to_account(fee_purse, caller, fee_purse_balance, None)
        .unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn deposit() {
    let owner: AccountHash = runtime::get_named_arg(constants::ARG_OWNER);
    let owner_balance = utils::get_storage_dic::<U512>(
        utils::get_uref(constants::KEY_OWNER_BALANCE),
        owner.to_string().as_str(),
    )
    .unwrap_or_revert_with(ApiError::from(Error::Unregistered));

    let purse = runtime::get_key(constants::KEY_PURSE)
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert();
    let purse_balance = system::get_purse_balance(purse).unwrap_or_revert();

    system::transfer_from_purse_to_purse(
        purse,
        utils::get_uref(constants::KEY_DEPOSIT_PURSE),
        purse_balance,
        None,
    )
    .unwrap_or_revert();

    utils::write_storage_dic(
        utils::get_uref(constants::KEY_OWNER_BALANCE),
        owner.to_string().as_str(),
        owner_balance + purse_balance,
    );
}

fn install_contract() {
    let name: String = runtime::get_named_arg(constants::ARG_NAME);
    if name.is_empty() {
        runtime::revert(ApiError::from(ApiError::InvalidArgument))
    }
    // Create the entry points for this contract.
    let mut entry_points = EntryPoints::new();
    load_entry_points(&mut entry_points);
    let mut named_keys = NamedKeys::new();
    named_keys.insert(
        constants::KEY_NAME.to_string(),
        storage::new_uref(name).into(),
    );
    named_keys.insert(
        constants::KEY_INSTALLER.to_string(),
        storage::new_uref(runtime::get_caller()).into(),
    );
    named_keys.insert(
        constants::KEY_FEE_RATE.to_string(),
        storage::new_uref(0u32).into(),
    );
    // Create a new contract package
    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(constants::CONTRACT_PACKAGE_NAME.to_string()),
        Some(constants::CONTRACT_ACCESS_UREF.to_string()),
    );

    // Store the contract version in the context's named keys.
    runtime::put_key(
        constants::CONTRACT_VERSION_KEY,
        storage::new_uref(contract_version).into(),
    );
    // Create a named key for the contract hash.
    runtime::put_key(constants::CONTRACT_KEY, contract_hash.into());

    // Call contract to initialize
    runtime::call_contract::<()>(contract_hash, constants::ENTRY_POINT_INIT, runtime_args! {});
}

fn upgrade_contract() {
    let contract_package_hash = runtime::get_key(constants::CONTRACT_PACKAGE_NAME)
        .unwrap_or_revert()
        .into_hash()
        .unwrap()
        .into();

    // Create the entry points for this contract.
    let mut entry_points = EntryPoints::new();
    load_entry_points(&mut entry_points);

    let (contract_hash, contract_version) =
        storage::add_contract_version(contract_package_hash, entry_points, NamedKeys::default());

    // Update contract hash and version
    runtime::put_key(
        constants::CONTRACT_VERSION_KEY,
        storage::new_uref(contract_version).into(),
    );
    runtime::put_key(constants::CONTRACT_KEY, contract_hash.into());
}

fn load_entry_points(entry_points: &mut EntryPoints) {
    entry_points.add_entry_point(EntryPoint::new(
        constants::ENTRY_POINT_INIT,
        Vec::new(),
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        constants::ENTRY_POINT_REGISTER,
        vec![
            Parameter::new(constants::ARG_CONTRACT, ContractHash::cl_type()),
            Parameter::new(constants::ARG_OWNER, AccountHash::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        constants::ENTRY_POINT_DEPOSIT,
        vec![
            Parameter::new(constants::ARG_OWNER, AccountHash::cl_type()),
            Parameter::new(constants::ARG_AMOUNT, U512::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        constants::ENTRY_POINT_CALL_ON_BEHALF,
        vec![
            Parameter::new(constants::ARG_ENTRY_POINT, CLType::String),
            Parameter::new(constants::ARG_CALLER, CLType::Key),
            Parameter::new(constants::ARG_ARGS, CLType::List(Box::new(CLType::Any))),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from(constants::ENTRY_POINT_GET_PURSE),
        Vec::new(),
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        constants::ENTRY_POINT_SET_FEE_RATE,
        vec![Parameter::new(constants::ARG_FEE_RATE, CLType::U512)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        constants::ENTRY_POINT_CLAIM_FEE,
        Vec::new(),
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
}
