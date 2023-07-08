contract;

use std::{
    auth::{
        AuthError,
        msg_sender,
    },
    call_frames::msg_asset_id,
    constants::BASE_ASSET_ID,
    context::msg_amount,
    token::transfer,
};

abi WalletContract {

    #[storage(write)]
    fn initialize_balance(value: u64) -> u64;

    #[storage(read)]
    fn read_balance() -> u64;

    #[storage(read, write), payable]
    fn receive_funds();

    #[storage(read, write)]
    fn send_funds_addr(amount_to_send: u64, recipient_address: Address);

    #[storage(read, write)]
    fn send_funds_iden(amount_to_send: u64, to_recip_ident: Identity);

}

// add your owners hex encoded address here (with the 0x this time):
// const OWNER_ADDRESS = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);

storage {
    balance: u64 = 0,
}

impl WalletContract for Contract {

    #[storage(write)]
    fn initialize_balance(value: u64) -> u64 {
        // make sure the only call to this function is from the owner.
        let sender: Result<Identity, AuthError> = msg_sender();
        match sender.unwrap() {
            Identity::Address(addr) => assert(addr == OWNER_ADDRESS),
            _ => revert(0),
        };
        storage.balance = value;
        value
    }

    #[storage(read)]
    fn read_balance() -> u64 {
       let bal = storage.balance;
       bal
    }

    #[storage(read, write), payable]
    fn receive_funds() {
        if msg_asset_id() == BASE_ASSET_ID {
            storage.balance += msg_amount();
        }
    }

    #[storage(read, write)]
    fn send_funds_addr(amount_to_send: u64, recipient_address: Address) {
        let sender: Result<Identity, AuthError> = msg_sender();
        match sender.unwrap() {
            Identity::Address(addr) => assert(addr == OWNER_ADDRESS),
            _ => revert(0),
        };
        let current_balance = storage.balance;
        assert(current_balance >= amount_to_send);
        storage.balance = current_balance - amount_to_send;
        let to_recip_addr = Identity::Address(recipient_address);
        transfer(amount_to_send, BASE_ASSET_ID, to_recip_addr);
    }

    #[storage(read, write)]
    fn send_funds_iden(amount_to_send: u64, to_recip_ident: Identity) {
        let sender: Result<Identity, AuthError> = msg_sender();
        match sender.unwrap() {
            Identity::Address(addr) => assert(addr == OWNER_ADDRESS),
            _ => revert(0),
        };
        let current_balance = storage.balance;
        assert(current_balance >= amount_to_send);
        storage.balance = current_balance - amount_to_send;
        transfer(amount_to_send, BASE_ASSET_ID, to_recip_ident);
    }

}

