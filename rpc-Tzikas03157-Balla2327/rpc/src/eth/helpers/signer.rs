//! An abstraction over Ethereum signers.

use std::collections::HashMap;

use alloy_dyn_abi::TypedData;
use reth_primitives::{
    eip191_hash_message, sign_message, Address, Signature, TransactionSigned, B256,
};
use reth_rpc_eth_api::helpers::{signer::Result, EthSigner};
use reth_rpc_eth_types::SignError;
use reth_rpc_types::TypedTransactionRequest;
use reth_rpc_types_compat::transaction::to_primitive_transaction;
use secp256k1::SecretKey;

/// Holds developer keys.
/// This struct is used to manage and utilize Ethereum addresses and their corresponding secret keys for signing purposes.
#[derive(Debug, Clone)]
pub struct DevSigner {
    addresses: Vec<Address>,
    accounts: HashMap<Address, SecretKey>,
}

#[allow(dead_code)]
impl DevSigner {
    /// Generates a random dev signer which satisfies the [`EthSigner`] trait.
    /// This function creates a single random signer and returns it as a boxed trait object.
    pub(crate) fn random() -> Box<dyn EthSigner> {
        let mut signers = Self::random_signers(1);
        signers.pop().expect("expect to generate at least one signer")
    }

    /// Generates the specified number of random dev signers which satisfy the [`EthSigner`] trait.
    /// This function creates a vector of random signers.
    pub(crate) fn random_signers(num: u32) -> Vec<Box<dyn EthSigner + 'static>> {
        let mut signers = Vec::new();
        for _ in 0..num {
            // Generate a random keypair using the secp256k1 library.
            let (sk, pk) = secp256k1::generate_keypair(&mut rand::thread_rng());

            // Convert the public key to an Ethereum address.
            let address = reth_primitives::public_key_to_address(pk);
            let addresses = vec![address];
            let accounts = HashMap::from([(address, sk)]);
            // Box the DevSigner and push it to the signers vector.
            signers.push(Box::new(Self { addresses, accounts }) as Box<dyn EthSigner>);
        }
        signers
    }

    /// Retrieves the secret key associated with the provided account address.
    /// Returns an error if the account is not found.
    fn get_key(&self, account: Address) -> Result<&SecretKey> {
        self.accounts.get(&account).ok_or(SignError::NoAccount)
    }

    /// Signs the given hash using the secret key associated with the provided account address.
    /// Returns the signature or an error if signing fails.
    fn sign_hash(&self, hash: B256, account: Address) -> Result<Signature> {
        let secret = self.get_key(account)?;
        let signature = sign_message(B256::from_slice(secret.as_ref()), hash);
        signature.map_err(|_| SignError::CouldNotSign)
    }
}

#[async_trait::async_trait]
impl EthSigner for DevSigner {
    /// Returns a list of addresses managed by this signer.
    fn accounts(&self) -> Vec<Address> {
        self.addresses.clone()
    }

    /// Checks if the signer can sign for the given address.
    fn is_signer_for(&self, addr: &Address) -> bool {
        self.accounts.contains_key(addr)
    }

    /// Signs a message asynchronously using the secret key associated with the provided address.
    /// The message is hashed according to EIP 191 before signing.
    async fn sign(&self, address: Address, message: &[u8]) -> Result<Signature> {
        // Hash message according to EIP 191:
        // https://ethereum.org/es/developers/docs/apis/json-rpc/#eth_sign
        let hash = eip191_hash_message(message);
        self.sign_hash(hash, address)
    }

    /// Signs a transaction request using the secret key associated with the provided address.
    /// Returns the signed transaction.
    fn sign_transaction(
        &self,
        request: TypedTransactionRequest,
        address: &Address,
    ) -> Result<TransactionSigned> {
        // Convert to primitive transaction.
        let transaction =
            to_primitive_transaction(request).ok_or(SignError::InvalidTransactionRequest)?;
        let tx_signature_hash = transaction.signature_hash();
        let signature = self.sign_hash(tx_signature_hash, *address)?;

        Ok(TransactionSigned::from_transaction_and_signature(transaction, signature))
    }

    /// Signs typed data using the secret key associated with the provided address.
    /// Returns the signature.
    fn sign_typed_data(&self, address: Address, payload: &TypedData) -> Result<Signature> {
        let encoded = payload.eip712_signing_hash().map_err(|_| SignError::InvalidTypedData)?;
        self.sign_hash(encoded, address)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use reth_primitives::U256;

    use super::*;

    /// Builds a DevSigner for testing purposes.
    fn build_signer() -> DevSigner {
        let addresses = vec![];
        let secret =
            SecretKey::from_str("4646464646464646464646464646464646464646464646464646464646464646")
                .unwrap();
        let accounts = HashMap::from([(Address::default(), secret)]);
        DevSigner { addresses, accounts }
    }

    /// Tests the signing of typed data using the DevSigner.
    #[tokio::test]
    async fn test_sign_type_data() {
        let eip_712_example = r#"{
            "types": {
            "EIP712Domain": [
                {
                    "name": "name",
                    "type": "string"
                },
                {
                    "name": "version",
                    "type": "string"
                },
                {
                    "name": "chainId",
                    "type": "uint256"
                },
                {
                    "name": "verifyingContract",
                    "type": "address"
                }
            ],
            "Person": [
                {
                    "name": "name",
                    "type": "string"
                },
                {
                    "name": "wallet",
                    "type": "address"
                }
            ],
            "Mail": [
                {
                    "name": "from",
                    "type": "Person"
                },
                {
                    "name": "to",
                    "type": "Person"
                },
                {
                    "name": "contents",
                    "type": "string"
                }
            ]
        },
        "primaryType": "Mail",
        "domain": {
            "name": "Ether Mail",
            "version": "1",
            "chainId": 1,
            "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
        },
        "message": {
            "from": {
                "name": "Cow",
                "wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
            },
            "to": {
                "name": "Bob",
                "wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
            },
            "contents": "Hello, Bob!"
        }
        }"#;
        let data: TypedData = serde_json::from_str(eip_712_example).unwrap();
        let signer = build_signer();
        let sig = signer.sign_typed_data(Address::default(), &data).unwrap();
        let expected = Signature {
            r: U256::from_str_radix(
                "5318aee9942b84885761bb20e768372b76e7ee454fc4d39b59ce07338d15a06c",
                16,
            )
            .unwrap(),
            s: U256::from_str_radix(
                "5e585a2f4882ec3228a9303244798b47a9102e4be72f48159d890c73e4511d79",
                16,
            )
            .unwrap(),
            odd_y_parity: false,
        };
        assert_eq!(sig, expected)
    }

    /// Tests the signing of a message using the DevSigner.
    #[tokio::test]
    async fn test_signer() {
        let message = b"Test message";
        let signer = build_signer();
        let sig = signer.sign(Address::default(), message).await.unwrap();
        let expected = Signature {
            r: U256::from_str_radix(
                "54313da7432e4058b8d22491b2e7dbb19c7186c35c24155bec0820a8a2bfe0c1",
                16,
            )
            .unwrap(),
            s: U256::from_str_radix(
                "687250f11a3d4435004c04a4cb60e846bc27997271d67f21c6c8170f17a25e10",
                16,
            )
            .unwrap(),
            odd_y_parity: true,
        };
        assert_eq!(sig, expected)
    }
}
