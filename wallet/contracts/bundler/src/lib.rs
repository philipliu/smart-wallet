#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, token, Address, Env, Symbol, Val};

#[contracttype]
pub enum DataKey {
    BundlerAccount,
    TokenAddress,
}

#[contracterror]
pub enum BundlerError {
    InsufficientFee = 1,
    InnerFailure = 2,
    TransferFailed = 3,
}


#[contract]
pub struct Bundler;

#[contractimpl]
impl Bundler {
    pub fn __constructor(env: Env, token_address: Address, bundler_account: Address) {
        env.storage().instance().set(&DataKey::BundlerAccount, &bundler_account);
        env.storage().instance().set(&DataKey::TokenAddress, &token_address);
    }

    pub fn execute(
        env: Env,
        caller: Address,
        max_fee: i128,
        simulated_fee: i128,
        contract_address: Address,
        function_name: Symbol,
        args: soroban_sdk::Vec<Val>,
    ) -> Result<soroban_sdk::Val, BundlerError> {
        let bundler_account: Address = env
            .storage()
            .instance()
            .get(&DataKey::BundlerAccount)
            .unwrap();
        bundler_account.require_auth();
        caller.require_auth();

        if simulated_fee > max_fee {
            return Err(BundlerError::InsufficientFee);
        }

        let res = env.try_invoke_contract::<soroban_sdk::Val, soroban_sdk::xdr::Error>(
            &contract_address,
            &function_name,
            args,
        ).unwrap();

        let token_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let token_client = token::TokenClient::new(&env, &token_address);

        if let Err(_) = token_client.try_transfer(&caller, &env.current_contract_address(), &max_fee) {
            return Err(BundlerError::TransferFailed);
        }

        res.map_err(|_| BundlerError::InnerFailure)
    }
}
