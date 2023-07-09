use fuels::{prelude::*,
    types::{ContractId, Identity},
    signers::fuel_crypto::SecretKey,
};
use rand::prelude::{Rng};
use dotenv::dotenv;
use std::str::FromStr;

pub const BASE_ASSET_ID: AssetId = AssetId::BASE;
// const RPC: &str = "127.0.0.1:4000";
const RPC: &str = "beta-3.fuel.network";

// Load abi from json
abigen!(Contract(
    name = "WalletContract",
    abi = "out/debug/basic_wallet_contract-abi.json"
));


//---------------------------------------------------------------------------------

#[tokio::test]
async fn convert_fuel_address_to_hex(){
    let _provider = match Provider::connect(RPC).await {
        Ok(p) => p,
        Err(error) => panic!("❌ Problem creating provider: {:#?}", error),
    };
    dotenv().ok();
    let secret0 = match std::env::var("OWNER_SECRET_KEY") {
        Ok(s) => s,
        Err(error) => panic!("❌ Cannot find OWNER_SECRET_KEY in .env file: {:#?}", error),
    };
    let wallet0 = WalletUnlocked::new_from_private_key(
        SecretKey::from_str(&secret0).unwrap(), Some(_provider.clone()));
    let secret1 = match std::env::var("ALICE_SECRET_KEY") {
        Ok(s) => s,
        Err(error) => panic!("❌ Cannot find ALICE_SECRET_KEY in .env file: {:#?}", error),
    };
    let wallet1 = WalletUnlocked::new_from_private_key(
        SecretKey::from_str(&secret1).unwrap(), Some(_provider.clone()));
    let secret2 = match std::env::var("BOB_SECRET_KEY") {
        Ok(s) => s,
        Err(error) => panic!("❌ Cannot find BOB_SECRET_KEY in .env file: {:#?}", error),
    };
    let wallet2 = WalletUnlocked::new_from_private_key(
        SecretKey::from_str(&secret2).unwrap(), Some(_provider.clone()));

    println!("Owner address:");
    println!("\tbech32 \t: {}", wallet0.address().to_string());
    println!("\t0x     \t: {}\n", Address::from(wallet0.address()));
    println!("Alice address:");
    println!("\tbech32 \t: {}", wallet1.address().to_string());
    println!("\t0x     \t: {}\n", Address::from(wallet1.address()));
    println!("Bob address:");
    println!("\tbech32 \t: {}", wallet2.address().to_string());
    println!("\t0x     \t: {}\n", Address::from(wallet2.address()));

}


async fn get_node_wallets(_p: &Provider) -> Vec<WalletUnlocked> {
    // get the private key for the SECRETX from the .env file in the project root dir:
    let mut wallets: Vec<WalletUnlocked> = Vec::new();
    wallets.push(get_wallet_from_env("OWNER_SECRET_KEY", _p.clone()));
    wallets.push(get_wallet_from_env("ALICE_SECRET_KEY", _p.clone()));
    wallets.push(get_wallet_from_env("BOB_SECRET_KEY", _p.clone()));
    wallets
}

async fn print_wallet_balances(_p: &Provider) {
    let mut wallets = get_node_wallets(_p).await;
    let wallet2 = wallets.pop().unwrap();
    let wallet1 = wallets.pop().unwrap();
    let wallet0 = wallets.pop().unwrap();
    let balance_base_w0 = wallet0.get_asset_balance(&BASE_ASSET_ID).await.unwrap();
    let balance_base_w1 = wallet1.get_asset_balance(&BASE_ASSET_ID).await.unwrap();
    let balance_base_w2 = wallet2.get_asset_balance(&BASE_ASSET_ID).await.unwrap();
    println!("Owner address (hex) \t: 0x{}", Address::from(wallet0.address()));
    println!("ETH: {:#?}", balance_base_w0 );
    println!("");
    println!("Alice address (hex) \t: 0x{}", Address::from(wallet1.address()));
    println!("ETH: {:#?}", balance_base_w1 );
    println!("");
    println!("Bob address (hex) \t: 0x{}", Address::from(wallet2.address()));
    println!("ETH: {:#?}", balance_base_w2 );
    println!("");
}

///
/// Print all the account BASE_ASSET balances and the Contract balances.
///
#[tokio::test]
async fn print_balances(){
    let provider = match Provider::connect(RPC).await {
        Ok(p) => p,
        Err(error) => panic!("❌ Problem creating provider: {:#?}", error),
    };
    println!("---------------------------------------------------------------------------");
    println!("balances:\n");

    // print the account balances:
    print_wallet_balances(&provider).await;

    dotenv().ok();
    let cid = match std::env::var("CONTRACTID") {
        Ok(s) => s,
        Err(error) => panic!("❌ Cannot find .env file: {:#?}", error),
    };
    let _contract_id: ContractId =
        cid.to_string()
        .parse()
        .expect("Invalid ID");

    let ca_hex = Address::from(*_contract_id.clone());
    let ca_bech32 = Bech32Address::from(ca_hex);
    println!("Contract ID (hex)    \t= {}", cid.to_string());
    println!("Contract ID (bech32) \t= {}", ca_bech32.to_string());

    let owner_wallet = get_wallet_from_env("OWNER_SECRET_KEY", provider.clone());
    let contract_instance = WalletContract::new(_contract_id.into(), owner_wallet);
    let contract_balances = contract_instance.get_balances().await;
    println!("\nAll Contract balances : {:#?}", contract_balances );
    println!("---------------------------------------------------------------------------\n");
}

fn get_wallet_from_env(env_sk: &str, provider: Provider) -> WalletUnlocked {
    dotenv().ok();
    let wallet = WalletUnlocked::new_from_private_key(
        SecretKey::from_str(
            &match std::env::var(env_sk) {
                Ok(s) => s,
                Err(error) => panic!("❌ Cannot find .env file: {:#?}", error),
            }
        ).unwrap(),
        Some(provider.clone())
    );
    wallet
}

#[allow(dead_code)]
async fn deploy_and_ret_wallets_instances() -> (Vec<WalletContract>, Vec<WalletUnlocked>) {

    let _provider = match Provider::connect(RPC).await {
        Ok(p) => p,
        Err(error) => panic!("❌ Problem creating provider: {:#?}", error),
    };
    let mut wallets = get_node_wallets(&_provider).await;
    let wallet2 = wallets.pop().unwrap();
    let wallet1 = wallets.pop().unwrap();
    let wallet0 = wallets.pop().unwrap();

    print_wallet_balances(&_provider).await;

    //-------------------------------------------
    // deploy with salt:
    let mut rng = rand::thread_rng();
    let salt = rng.gen::<[u8; 32]>();

    let tx_parameters = TxParameters::default()
        .set_gas_price(1)
        .set_gas_limit(1_000_000)
        .set_maturity(0);

    let c_id = Contract::deploy(
        "./out/debug/basic_wallet_contract.bin",
        &wallet0,
        DeployConfiguration::default()
            .set_salt(salt)
            .set_tx_parameters(tx_parameters),
        )
        .await;

    let contract_id = match c_id {
        Ok(contractid) => contractid,
        Err(error) => panic!("❌ Problem deploying the contract: {:#?}", error),
    };
    println!("Contract deployed:");
    println!("contract ID bech32 \t: {}", contract_id.clone().to_string());
    let bcontract_id: ContractId = contract_id.clone().into();
    println!("contract ID (hex) \t: 0x{}", bcontract_id);

    let contract_instance0 = WalletContract::new(contract_id.clone(), wallet0.clone());
    let contract_instance1 = WalletContract::new(contract_id.clone(), wallet1.clone());
    let contract_instance2 = WalletContract::new(contract_id.clone(), wallet2.clone());
    let mut contract_instances: Vec<WalletContract> = Vec::new();
    contract_instances.push(contract_instance2);
    contract_instances.push(contract_instance1);
    contract_instances.push(contract_instance0);
    (contract_instances, wallets)
}

#[tokio::test]
async fn deploy() {
    // uncomment to sue this function to deploy.
    //let (_con_insts, _wals) = deploy_and_ret_wallets_instances().await;
}


//--------------------------------------------------------------------------
// Main Test Functions:

///
/// # Initalize the contract asset balance
///
#[tokio::test]
async fn initalize_contract_balance() {
    println!("Initalize the contract asset balance:");

    let provider = match Provider::connect(RPC).await {
        Ok(p) => p,
        Err(error) => panic!("❌ Problem creating provider: {:#?}", error),
    };

    dotenv().ok();
    let cid = match std::env::var("CONTRACTID") {
        Ok(s) => s,
        Err(error) => panic!("❌ Cannot find CONTRACTID in .env file: {:#?}", error),
    };

    let _contract_id: ContractId =
        cid.to_string()
        .parse()
        .expect("Invalid ID");

    println!("Contract ID \t= {}", Address::from(*_contract_id.clone()));

    // get the owners secret key/wallet
    let owner_wallet = get_wallet_from_env("OWNER_SECRET_KEY", provider.clone());

    let contract_instance = WalletContract::new(_contract_id.into(), owner_wallet);
    let tx_params = TxParameters::default()
        .set_gas_price(1)
        .set_gas_limit(1_000_000)
        .set_maturity(0);

    let result = contract_instance
        .methods()
        .initialize_balance(0)
        .tx_params(tx_params)
        .call()
        .await;

    let response = match result{
    Ok(v) => v,
    Err(error) => panic!("Problem calling contract initialize_balance() --> {:?}", error),
    };
    assert_eq!(<u64 as Into<u64>>::into(0u64), response.value);
}


///
/// # Read contract assets balances and storage.balance
///
#[tokio::test]
async fn read_contract_storage_and_asset_balances() {
    println!("Read contract assets balances and storage.balance:");

    let provider = match Provider::connect(RPC).await {
        Ok(p) => p,
        Err(error) => panic!("❌ Problem creating provider: {:#?}", error),
    };

    dotenv().ok();
    let cid = match std::env::var("CONTRACTID") {
        Ok(s) => s,
        Err(error) => panic!("❌ Cannot find .env file: {:#?}", error),
    };

    let _contract_id: ContractId =
        cid.to_string()
        .parse()
        .expect("Invalid ID");
    println!("Contract id = {}", Address::from(*_contract_id.clone()));

    // get user wallet
    let user1_wallet = get_wallet_from_env("ALICE_SECRET_KEY", provider.clone());

    let contract_instance = WalletContract::new(_contract_id.into(), user1_wallet);

    let tx_params = TxParameters::default()
        .set_gas_price(1)
        .set_gas_limit(1_000_000)
        .set_maturity(0);

    let result = contract_instance
        .methods()
        .read_balance()
        .tx_params(tx_params)
        .call()
        .await;
    println!("{} read_balance (on {})", if result.is_ok() { "✅" } else { "❌" }, RPC.to_string());
    println!("read storage.balance value = {}", result.unwrap().value);
    let contract_balances = contract_instance.get_balances().await;
    println!("\nAll contract balances : {:#?}", contract_balances );

}

///
/// # Alice to send BASE_ASSET into contract.
///
/// using the receive_funds() function & signing \
/// the transaction with Alices secret key.
///
#[tokio::test]
async fn send_from_alice_to_contract() {
    println!("Send into contract using receive_funds:");

    let provider = match Provider::connect(RPC).await {
        Ok(p) => p,
        Err(error) => panic!("❌ Problem creating provider: {:#?}", error),
    };

    dotenv().ok();
    let cid = match std::env::var("CONTRACTID") {
        Ok(s) => s,
        Err(error) => panic!("❌ Cannot find .env file: {:#?}", error),
    };

    let _contract_id: ContractId =
        cid.to_string()
        .parse()
        .expect("Invalid ID");
    println!("Contract id = {}", Address::from(*_contract_id.clone()));

    //------------------------------------
    // get Alice wallet
    let user1_wallet = get_wallet_from_env("ALICE_SECRET_KEY", provider.clone());

    println!("---------------------------------------------------------------------------");
    let wal1_base_bal_start = user1_wallet.clone().get_asset_balance(&BASE_ASSET_ID).await.unwrap();
    println!("Alice ETH balance before \t= {}", wal1_base_bal_start );

    let contract_instance = WalletContract::new(_contract_id.into(), user1_wallet.clone());
    let tx_params = TxParameters::default()
        .set_gas_price(1)
        .set_gas_limit(1_000_000)
        .set_maturity(0);

    //let deposit_amount = 2_000_005;
    let deposit_amount = 499_999_000;

    let call_params = CallParameters::default()
        .set_amount(deposit_amount)
        .set_asset_id(BASE_ASSET_ID);

    let response1 = contract_instance
        .methods()
        .receive_funds()
        .tx_params(tx_params)
        .call_params(call_params).unwrap()
        .call()
        .await;

    let response = match response1{
        Ok(v) => v,
        Err(error) => panic!("Problem calling contract () --> {:#?}", error),
    };

    println!("\nGas used in tx \t= {}\n", response.gas_used);

    let wal1_base_bal_end = user1_wallet.get_asset_balance(&BASE_ASSET_ID).await.unwrap();
    println!("Alice ETH balance after \t= {}", wal1_base_bal_end );
    println!("---------------------------------------------------------------------------");


}

///
/// # Send BASE_ASSET to Bob using Identity
///
/// This function will send from the WalletContract using
/// the owners secret key to sign the transaction, an amount:
///
/// amount = 1_000_000
///
/// within,
///
/// .send_funds_iden(1_000_000, to_identity)
///
/// to Bobs Identity constructed from the encoded public key 0x...:
///
/// to_identity = Identity::Address(recipient_base_layer_address.into());
///
/// recipient_base_layer_address = 0x<Bobs address>
///
///
#[tokio::test]
async fn send_base_asset_to_bob_using_identity() {
    println!("Send base_asset to Bob using Identity:");

    let provider = match Provider::connect(RPC).await {
        Ok(p) => p,
        Err(error) => panic!("❌ Problem creating provider: {:#?}", error),
    };

    dotenv().ok();
    let cid = match std::env::var("CONTRACTID") {
        Ok(s) => s,
        Err(error) => panic!("❌ Cannot find .env file: {:#?}", error),
    };
    let _contract_id: ContractId =
        cid.to_string()
        .parse()
        .expect("Invalid ID");

    println!("contract ID = {}", Address::from(*_contract_id.clone()));

    let owner_wallet = get_wallet_from_env("OWNER_SECRET_KEY", provider);
    let contract_instance = WalletContract::new(_contract_id.into(), owner_wallet);

    let tx_params = TxParameters::default()
        .set_gas_price(1)
        .set_gas_limit(1_000_000)
        .set_maturity(0);

    //-------------------------------------------------------------
    // Address for the recipient (remove the 0x)

    let recipient_base_layer_address =
    Address::from_str("3ea052590cf8c1b91361e997685972332c8925bb96dd2b8bb9ca2f9c03d33645")
        .expect("Invalid address.");

    let to_identity = Identity::Address(recipient_base_layer_address.into());

    let result = contract_instance
        .methods()
        .send_funds_iden(1_000_000, to_identity)
        .append_variable_outputs(1)
        .tx_params(tx_params)
        .call()
        .await;

    match result {
        // The transaction is valid and executes to completion
        Ok(call_response) => {
            let _cr = call_response;
            println!("\n✅ send successful to 0x{:?}", recipient_base_layer_address);
        }
        // The transaction is malformed
        Err(Error::ValidationError(e)) => {
            println!("\nTransaction is malformed (ValidationError): {e}");
        }
        // Failed request to provider
        Err(Error::ProviderError(reason)) => {
            println!("\nProvider request failed with reason: {reason}");
        }
        // The transaction is valid but reverts
        Err(Error::RevertTransactionError {
            reason, receipts, ..
        }) => {
            println!("\nContractCall failed with reason: {reason}");
            println!("Transaction receipts are: {receipts:#?}");
            panic!("Problem calling contract --> The transaction is valid but reverts. ");
        }
        Err(_) => {}
    }

}


///
/// # Send BASE_ASSET to Bob using Address
///
/// This function will send from the WalletContract using
/// the owners secret key to sign the transaction, an amount:
///
/// amount = 1_000_000
///
/// within,
///
/// .send_funds_addr(1_000_000, recipient_base_layer_address)
///
/// to Bob hex encoded public key 0x...:
///
/// recipient_base_layer_address = 0x<Bobs address>
///
///
#[tokio::test]
async fn send_base_asset_to_bob_using_address() {
    println!("Send base_asset to Bob using Address:");

    let provider = match Provider::connect(RPC).await {
        Ok(p) => p,
        Err(error) => panic!("❌ Problem creating provider: {:#?}", error),
    };

    dotenv().ok();
    let cid = match std::env::var("CONTRACTID") {
        Ok(s) => s,
        Err(error) => panic!("❌ Cannot find .env file: {:#?}", error),
    };
    let _contract_id: ContractId =
        cid.to_string()
        .parse()
        .expect("Invalid ID");

    println!("contract ID = {}", Address::from(*_contract_id.clone()));

    let owner_wallet = get_wallet_from_env("OWNER_SECRET_KEY", provider);
    let contract_instance = WalletContract::new(_contract_id.into(), owner_wallet);

    let tx_params = TxParameters::default()
        .set_gas_price(1)
        .set_gas_limit(1_000_000)
        .set_maturity(0);

    //-------------------------------------------------------------
    // Address for the recipient (remove the 0x)

    let recipient_base_layer_address =
    Address::from_str("3ea052590cf8c1b91361e997685972332c8925bb96dd2b8bb9ca2f9c03d33645")
        .expect("Invalid address.");

    let result = contract_instance
        .methods()
        .send_funds_addr(1_000_000, recipient_base_layer_address)
        .append_variable_outputs(1)
        .tx_params(tx_params)
        .call()
        .await;

    match result {
        // The transaction is valid and executes to completion
        Ok(call_response) => {
            let _cr = call_response;
            println!("\n✅ send successful to 0x{:?}", recipient_base_layer_address);
        }
        // The transaction is malformed
        Err(Error::ValidationError(e)) => {
            println!("\nTransaction is malformed (ValidationError): {e}");
        }
        // Failed request to provider
        Err(Error::ProviderError(reason)) => {
            println!("\nProvider request failed with reason: {reason}");
        }
        // The transaction is valid but reverts
        Err(Error::RevertTransactionError {
            reason, receipts, ..
        }) => {
            println!("\nContractCall failed with reason: {reason}");
            println!("Transaction receipts are: {receipts:#?}");
            panic!("Problem calling contract --> The transaction is valid but reverts. ");
        }
        Err(_) => {}
    }

}


