use crate::constants;
use alloc::borrow::ToOwned;
use casper_contract::contract_api::storage;
use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casper_types::account::AccountHash;
use casper_types::bytesrepr::{FromBytes, ToBytes};
use casper_types::system::CallStackElement;
use casper_types::{ApiError, CLTyped, ContractPackageHash, URef};

#[inline]
pub(crate) fn get_uref(key: &str) -> URef {
    let key = runtime::get_key(key)
        .ok_or(ApiError::MissingKey)
        .unwrap_or_revert();
    key.into_uref().unwrap_or_revert()
}

#[inline]
pub(crate) fn write_storage<T: CLTyped + ToBytes>(name: &str, value: T) {
    storage::write(get_uref(name), value);
}

#[inline]
pub(crate) fn get_storage<T: CLTyped + FromBytes>(name: &str) -> T {
    storage::read(get_uref(name))
        .unwrap_or_revert()
        .unwrap_or_revert()
}

#[inline]
pub(crate) fn get_caller() -> AccountHash {
    match *runtime::get_call_stack()
        .iter()
        .nth_back(1)
        .to_owned()
        .unwrap_or_revert()
    {
        CallStackElement::StoredContract {
            contract_package_hash,
            ..
        } => {
            // Check if called from relay contract
            if contract_package_hash
                == get_storage::<ContractPackageHash>(constants::KEY_RELAY_CONTRACT_PACKAGE)
            {
                return runtime::get_named_arg::<AccountHash>(constants::ARG_CALLER);
            }
            runtime::get_caller()
        }
        _ => runtime::get_caller(),
    }
}
