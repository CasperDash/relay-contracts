use casper_contract::contract_api::storage;
use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casper_types::bytesrepr::{FromBytes, ToBytes};
use casper_types::{ApiError, CLTyped, URef};

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
