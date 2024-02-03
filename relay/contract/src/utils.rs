use alloc::borrow::ToOwned;
use casper_contract::contract_api::storage;
use casper_contract::{contract_api::runtime, ext_ffi, unwrap_or_revert::UnwrapOrRevert};
use casper_types::account::AccountHash;
use casper_types::bytesrepr::{FromBytes, ToBytes};
use casper_types::system::CallStackElement;
use casper_types::{api_error, ApiError, CLTyped, ContractPackageHash, URef};

#[inline]
pub(crate) fn get_uref(key: &str) -> URef {
    let key = runtime::get_key(key)
        .ok_or(ApiError::MissingKey)
        .unwrap_or_revert();
    key.into_uref().unwrap_or_revert()
}

#[inline]
pub(crate) fn get_storage<T: CLTyped + FromBytes>(name: &str) -> T {
    storage::read(get_uref(name))
        .unwrap_or_revert()
        .unwrap_or_revert()
}

#[inline]
pub(crate) fn write_storage<T: CLTyped + ToBytes>(name: &str, value: T) {
    storage::write(get_uref(name), value);
}

#[inline]
pub(crate) fn get_storage_dic<T: CLTyped + FromBytes + ToBytes>(dic: URef, key: &str) -> Option<T> {
    storage::dictionary_get(dic, key).unwrap_or(None)
}

#[inline]
pub(crate) fn write_storage_dic<T: CLTyped + FromBytes + ToBytes>(dic: URef, key: &str, value: T) {
    storage::dictionary_put(dic, key, value);
}

pub fn get_named_arg_size(name: &str) -> Option<usize> {
    let mut arg_size: usize = 0;
    let ret = unsafe {
        ext_ffi::casper_get_named_arg_size(
            name.as_bytes().as_ptr(),
            name.len(),
            &mut arg_size as *mut usize,
        )
    };
    match api_error::result_from(ret) {
        Ok(_) => Some(arg_size),
        Err(ApiError::MissingArgument) => None,
        Err(e) => runtime::revert(e),
    }
}

#[inline]
pub fn get_optional_named_arg<T: FromBytes>(name: &str) -> Option<T> {
    get_named_arg_size(name)?;
    Some(runtime::get_named_arg::<T>(name))
}

#[inline]
pub(crate) fn get_contract_package() -> Option<ContractPackageHash> {
    match *runtime::get_call_stack()
        .last()
        .to_owned()
        .unwrap_or_revert()
    {
        CallStackElement::StoredContract {
            contract_package_hash,
            ..
        } => Some(contract_package_hash),
        _ => None,
    }
}
