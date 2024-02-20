use alloc::string::{String, ToString};
use casper_event_standard::Event;
use casper_types::account::AccountHash;
use casper_types::{ContractHash, U512};

#[derive(Event)]
pub struct Register {
    contract_hash: String,
    owner: String,
}

#[derive(Event)]
pub struct Deposit {
    owner: String,
    amount: String,
}

#[derive(Event)]
pub struct CallOnBehalf {
    contract_hash: String,
    owner: String,
    caller: String,
    entry_point: String,
    gas_amount: String,
    cep18_hash: Option<String>,
}

impl Register {
    pub fn new(contract_hash: ContractHash, owner: AccountHash) -> Self {
        Register {
            contract_hash: contract_hash.to_formatted_string(),
            owner: owner.to_formatted_string(),
        }
    }
}

impl Deposit {
    pub fn new(owner: AccountHash, amount: U512) -> Self {
        Deposit {
            owner: owner.to_formatted_string(),
            amount: amount.to_string(),
        }
    }
}

impl CallOnBehalf {
    pub fn new(
        contract_hash: ContractHash,
        owner: AccountHash,
        caller: AccountHash,
        entry_point: String,
        gas_amount: U512,
        cep18_hash: Option<ContractHash>,
    ) -> Self {
        CallOnBehalf {
            contract_hash: contract_hash.to_formatted_string(),
            owner: owner.to_formatted_string(),
            caller: caller.to_formatted_string(),
            entry_point,
            gas_amount: gas_amount.to_string(),
            cep18_hash: cep18_hash.map(|hash| hash.to_formatted_string()),
        }
    }
}
