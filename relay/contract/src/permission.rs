use crate::errors::Error;
use crate::{constants, utils};
use casper_contract::contract_api::runtime;
use casper_types::account::AccountHash;

pub enum Permission {
    Installer,
}

pub(crate) fn require(permission: Permission) {
    let caller = runtime::get_caller();
    match permission {
        Permission::Installer => {
            let installer = utils::get_storage::<AccountHash>(constants::KEY_INSTALLER);
            if caller != installer {
                runtime::revert(Error::Unauthorized);
            }
        }
    }
}
