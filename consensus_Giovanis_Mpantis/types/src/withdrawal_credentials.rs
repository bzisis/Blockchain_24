use crate::*;
use bls::get_withdrawal_credentials;

/// Represents withdrawal credentials for a validator.
pub struct WithdrawalCredentials(Hash256);

impl WithdrawalCredentials {
    /// Generates BLS withdrawal credentials using the provided public key and chain spec.
    ///
    /// # Arguments
    ///
    /// * `withdrawal_public_key` - The BLS public key of the validator.
    /// * `spec` - The chain specification defining parameters for generating credentials.
    ///
    /// # Returns
    ///
    /// A `WithdrawalCredentials` instance based on the BLS public key.
    pub fn bls(withdrawal_public_key: &PublicKey, spec: &ChainSpec) -> Self {
        let withdrawal_credentials =
            get_withdrawal_credentials(withdrawal_public_key, spec.bls_withdrawal_prefix_byte);
        Self(Hash256::from_slice(&withdrawal_credentials))
    }

    /// Generates ETH1 withdrawal credentials using the provided address and chain spec.
    ///
    /// # Arguments
    ///
    /// * `withdrawal_address` - The ETH1 address of the validator.
    /// * `spec` - The chain specification defining parameters for generating credentials.
    ///
    /// # Returns
    ///
    /// A `WithdrawalCredentials` instance based on the ETH1 address.
    pub fn eth1(withdrawal_address: Address, spec: &ChainSpec) -> Self {
        let mut withdrawal_credentials = [0; 32];
        withdrawal_credentials[0] = spec.eth1_address_withdrawal_prefix_byte;
        withdrawal_credentials[12..].copy_from_slice(withdrawal_address.as_bytes());
        Self(Hash256::from_slice(&withdrawal_credentials))
    }
}

impl From<WithdrawalCredentials> for Hash256 {
    /// Converts `WithdrawalCredentials` into `Hash256`.
    ///
    /// # Arguments
    ///
    /// * `withdrawal_credentials` - The withdrawal credentials to convert.
    ///
    /// # Returns
    ///
    /// A `Hash256` representation of the withdrawal credentials.
    fn from(withdrawal_credentials: WithdrawalCredentials) -> Self {
        withdrawal_credentials.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::generate_deterministic_keypair;
    use std::str::FromStr;

    #[test]
    fn bls_withdrawal_credentials() {
        let spec = &MainnetEthSpec::default_spec();
        let keypair = generate_deterministic_keypair(0);
        let credentials = WithdrawalCredentials::bls(&keypair.pk, spec);
        let manually_generated_credentials =
            get_withdrawal_credentials(&keypair.pk, spec.bls_withdrawal_prefix_byte);
        let hash: Hash256 = credentials.into();
        assert_eq!(hash[0], spec.bls_withdrawal_prefix_byte);
        assert_eq!(hash.as_bytes(), &manually_generated_credentials);
    }

    #[test]
    fn eth1_withdrawal_credentials() {
        let spec = &MainnetEthSpec::default_spec();
        let address = Address::from_str("0x25c4a76E7d118705e7Ea2e9b7d8C59930d8aCD3b").unwrap();
        let credentials = WithdrawalCredentials::eth1(address, spec);
        let hash: Hash256 = credentials.into();
        assert_eq!(
            hash,
            Hash256::from_str("0x01000000000000000000000025c4a76E7d118705e7Ea2e9b7d8C59930d8aCD3b")
                .unwrap()
        )
    }
}
