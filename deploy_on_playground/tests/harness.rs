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
    name = "MyContract",
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
    let secret0 = match std::env::var("SECRET0") {
        Ok(s) => s,
        Err(error) => panic!("❌ Cannot find SECRET0 in .env file: {:#?}", error),
    };
    let wallet0 = WalletUnlocked::new_from_private_key(
        SecretKey::from_str(&secret0).unwrap(), Some(_provider.clone()));
    let secret1 = match std::env::var("SECRET1") {
        Ok(s) => s,
        Err(error) => panic!("❌ Cannot find SECRET1 in .env file: {:#?}", error),
    };
    let wallet1 = WalletUnlocked::new_from_private_key(
        SecretKey::from_str(&secret1).unwrap(), Some(_provider.clone()));
    let secret2 = match std::env::var("SECRET2") {
        Ok(s) => s,
        Err(error) => panic!("❌ Cannot find SECRET2 in .env file: {:#?}", error),
    };
    let wallet2 = WalletUnlocked::new_from_private_key(
        SecretKey::from_str(&secret2).unwrap(), Some(_provider.clone()));

    println!("Owner address:");
    println!("\tbech32 \t: {}", wallet0.address().to_string());
    println!("\t0x     \t: {}\n", Address::from(wallet0.address()));
    println!("User1 address:");
    println!("\tbech32 \t: {}", wallet1.address().to_string());
    println!("\t0x     \t: {}\n", Address::from(wallet1.address()));
    println!("User2 address:");
    println!("\tbech32 \t: {}", wallet2.address().to_string());
    println!("\t0x     \t: {}\n", Address::from(wallet2.address()));

}


async fn get_node_wallets(_p: &Provider) -> Vec<WalletUnlocked> {
    // get the private key for the SECRETX from the .env file in the project root dir:
    let mut wallets: Vec<WalletUnlocked> = Vec::new();
    wallets.push(get_wallet_from_env("SECRET0", _p.clone()));
    wallets.push(get_wallet_from_env("SECRET1", _p.clone()));
    wallets.push(get_wallet_from_env("SECRET2", _p.clone()));
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
    println!("User1 address (hex) \t: 0x{}", Address::from(wallet1.address()));
    println!("ETH: {:#?}", balance_base_w1 );
    println!("");
    println!("User2 address (hex) \t: 0x{}", Address::from(wallet2.address()));
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

    let owner_wallet = get_wallet_from_env("SECRET0", provider.clone());
    let contract_instance = MyContract::new(_contract_id.into(), owner_wallet);
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

async fn deploy_and_ret_wallets_instances() -> (Vec<MyContract>, Vec<WalletUnlocked>) {

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

    let contract_instance0 = MyContract::new(contract_id.clone(), wallet0.clone());
    let contract_instance1 = MyContract::new(contract_id.clone(), wallet1.clone());
    let contract_instance2 = MyContract::new(contract_id.clone(), wallet2.clone());
    let mut contract_instances: Vec<MyContract> = Vec::new();
    contract_instances.push(contract_instance2);
    contract_instances.push(contract_instance1);
    contract_instances.push(contract_instance0);

    (contract_instances, wallets)
}

#[tokio::test]
async fn deploy() {
    let (_con_insts, _wals) = deploy_and_ret_wallets_instances().await;
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
    let owner_wallet = get_wallet_from_env("SECRET0", provider.clone());

    let contract_instance = MyContract::new(_contract_id.into(), owner_wallet);
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
    // println!("cid         = {}", cid.to_string());

    let _contract_id: ContractId =
        cid.to_string()
        .parse()
        .expect("Invalid ID");

    // println!("contract id = {}", Address::from(*_contract_id.clone()));

    // get user wallet
    let user1_wallet = get_wallet_from_env("SECRET1", provider.clone());

    let contract_instance = MyContract::new(_contract_id.into(), user1_wallet);

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
/// # Send into contract using receive_funds
///
#[tokio::test]
async fn send_from_user1_to_contract() {
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
    println!("cid         = {}", cid.to_string());

    let _contract_id: ContractId =
        cid.to_string()
        .parse()
        .expect("Invalid ID");
    println!("contract id = {}", Address::from(*_contract_id.clone()));

    // let ca_hex = Address::from(*_contract_id.clone());
    // let ca_bech32 = Bech32Address::from(ca_hex);


    //------------------------------------
    // get user1 wallet
    let user1_wallet = get_wallet_from_env("SECRET1", provider.clone());

    println!("---------------------------------------------------------------------------");
    let wal1_base_bal_start = user1_wallet.clone().get_asset_balance(&BASE_ASSET_ID).await.unwrap();
    println!("User1 ETH balance before \t= {}", wal1_base_bal_start );
    println!("---------------------------------------------------------------------------");

    let contract_instance = MyContract::new(_contract_id.into(), user1_wallet.clone());
    let tx_params = TxParameters::default()
        .set_gas_price(1)
        .set_gas_limit(1_000_000)
        .set_maturity(0);
    //let tx_params = TxParameters::default();

    let deposit_amount = 1_000_005;

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

    println!("gas used in tx \t= {}", response.gas_used);

    println!("---------------------------------------------------------------------------");
    let wal1_base_bal_end = user1_wallet.get_asset_balance(&BASE_ASSET_ID).await.unwrap();
    println!("User1 ETH balance after \t= {}", wal1_base_bal_end );
    println!("---------------------------------------------------------------------------");


}

///
/// # Send base_asset to User2 using Identity
///
#[tokio::test]
async fn send_base_asset_to_user2_using_identity() {
    println!("Send base_asset to User2 using Identity:");

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

    let owner_wallet = get_wallet_from_env("SECRET0", provider);
    let contract_instance = MyContract::new(_contract_id.into(), owner_wallet);

    let tx_params = TxParameters::default()
        .set_gas_price(1)
        .set_gas_limit(1_000_000)
        .set_maturity(0);


    //-------------------------------------------------------------
    // Address for the recipient (remove the 0x)

    let recipient_base_layer_address =
    Address::from_str("9a58e07905f7a01e3787a89a451b48144bb9d3d6f725402c89a18fd7f6033c6d")
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
/// # Send base_asset to User2 using Address
///
#[tokio::test]
async fn send_base_asset_to_user2_using_address() {
    println!("Send base_asset to User2 using Address:");

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

    let owner_wallet = get_wallet_from_env("SECRET0", provider);
    let contract_instance = MyContract::new(_contract_id.into(), owner_wallet);

    let tx_params = TxParameters::default()
        .set_gas_price(1)
        .set_gas_limit(1_000_000)
        .set_maturity(0);


    //-------------------------------------------------------------
    // Address for the recipient (remove the 0x)

    let recipient_base_layer_address =
    Address::from_str("9a58e07905f7a01e3787a89a451b48144bb9d3d6f725402c89a18fd7f6033c6d")
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


