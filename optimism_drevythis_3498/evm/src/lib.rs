//! EVM config for vanilla optimism.
//! This module provides configuration and utilities specific to Optimism EVM.

// HTML metadata for documentation purposes
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]

// Enable specific features if the `docsrs` feature is enabled
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

// This crate is only usable when compiled with the `optimism` feature enabled
#![cfg(feature = "optimism")]

// Import necessary crates and modules
use reth_chainspec::ChainSpec;
use reth_evm::{ConfigureEvm, ConfigureEvmEnv};
use reth_primitives::{
    revm_primitives::{AnalysisKind, CfgEnvWithHandlerCfg, TxEnv},
    transaction::FillTxEnv,
    Address, Head, Header, TransactionSigned, U256,
};
use reth_revm::{inspector_handle_register, Database, Evm, EvmBuilder, GetInspector};

// Import local modules and re-export them
mod config;
pub use config::{revm_spec, revm_spec_by_timestamp_after_bedrock};
mod execute;
pub use execute::*;
pub mod l1;
pub use l1::*;

mod error;
pub use error::OptimismBlockExecutionError;
use revm_primitives::{Bytes, Env, OptimismFields, TxKind};

/// Optimism-related EVM configuration.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct OptimismEvmConfig;

// Implementation of `ConfigureEvmEnv` for `OptimismEvmConfig`
impl ConfigureEvmEnv for OptimismEvmConfig {
    // Fill transaction environment for regular transactions
    fn fill_tx_env(&self, tx_env: &mut TxEnv, transaction: &TransactionSigned, sender: Address) {
        transaction.fill_tx_env(tx_env, sender);
    }

    // Fill transaction environment for system contract calls
    fn fill_tx_env_system_contract_call(
        &self,
        env: &mut Env,
        caller: Address,
        contract: Address,
        data: Bytes,
    ) {
        env.tx = TxEnv {
            caller,
            transact_to: TxKind::Call(contract),
            nonce: None,  // Disable nonce checks
            gas_limit: 30_000_000,
            value: U256::ZERO,
            data,
            gas_price: U256::ZERO,  // Zero gas price for system calls
            chain_id: None,  // No chain ID check
            gas_priority_fee: None,  // No priority fee
            access_list: Vec::new(),
            blob_hashes: Vec::new(),
            max_fee_per_blob_gas: None,
            optimism: OptimismFields {
                source_hash: None,
                mint: None,
                is_system_transaction: Some(false),  // Not a system transaction
                enveloped_tx: Some(Bytes::default()),  // EIP-4788: zero bytes for enveloped tx size
            },
        };

        env.block.gas_limit = U256::from(env.tx.gas_limit);  // Set block gas limit
        env.block.basefee = U256::ZERO;  // Disable base fee check
    }

    // Fill configuration environment
    fn fill_cfg_env(
        &self,
        cfg_env: &mut CfgEnvWithHandlerCfg,
        chain_spec: &ChainSpec,
        header: &Header,
        total_difficulty: U256,
    ) {
        let spec_id = revm_spec(
            chain_spec,
            &Head {
                number: header.number,
                timestamp: header.timestamp,
                difficulty: header.difficulty,
                total_difficulty,
                hash: Default::default(),
            },
        );

        cfg_env.chain_id = chain_spec.chain().id();  // Set chain ID
        cfg_env.perf_analyse_created_bytecodes = AnalysisKind::Analyse;  // Enable bytecode analysis

        cfg_env.handler_cfg.spec_id = spec_id;  // Set spec ID
        cfg_env.handler_cfg.is_optimism = chain_spec.is_optimism();  // Check if chain is Optimism-compatible
    }
}

// Implementation of `ConfigureEvm` for `OptimismEvmConfig`
impl ConfigureEvm for OptimismEvmConfig {
    type DefaultExternalContext<'a> = ();

    // Create an EVM instance
    fn evm<'a, DB: Database + 'a>(&self, db: DB) -> Evm<'a, Self::DefaultExternalContext<'a>, DB> {
        EvmBuilder::default().with_db(db).optimism().build()
    }

    // Create an EVM instance with an inspector
    fn evm_with_inspector<'a, DB, I>(&self, db: DB, inspector: I) -> Evm<'a, I, DB>
    where
        DB: Database + 'a,
        I: GetInspector<DB>,
    {
        EvmBuilder::default()
            .with_db(db)
            .with_external_context(inspector)
            .optimism()
            .append_handler_register(inspector_handle_register)
            .build()
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use reth_primitives::revm_primitives::{BlockEnv, CfgEnv};
    use revm_primitives::SpecId;

    #[test]
    #[ignore]
    fn test_fill_cfg_and_block_env() {
        // Setup test environment
        let mut cfg_env = CfgEnvWithHandlerCfg::new_with_spec_id(CfgEnv::default(), SpecId::LATEST);
        let mut block_env = BlockEnv::default();
        let header = Header::default();
        let chain_spec = ChainSpec::default();
        let total_difficulty = U256::ZERO;

        // Test `fill_cfg_and_block_env` method
        OptimismEvmConfig::default().fill_cfg_and_block_env(
            &mut cfg_env,
            &mut block_env,
            &chain_spec,
            &header,
            total_difficulty,
        );

        // Assertion: Check if chain ID is correctly set
        assert_eq!(cfg_env.chain_id, chain_spec.chain().id());
    }
}
