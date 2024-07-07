use ethereum_hashing::hash_fixed;
use types::{KzgCommitment, VersionedHash, VERSIONED_HASH_VERSION_KZG};

/// Converts a KZG commitment to a versioned hash.
///
/// This function takes a reference to a `KzgCommitment` and computes a versioned hash
/// using the Ethereum hashing algorithm `hash_fixed`. It then sets the first byte of
/// the hashed commitment to the version constant `VERSIONED_HASH_VERSION_KZG` and returns
/// the result as a `VersionedHash`.
///
/// # Arguments
///
/// * `kzg_commitment` - A reference to a `KzgCommitment`, which is a tuple struct containing
///                      data related to KZG commitments.
///
/// # Returns
///
/// A `VersionedHash` representing the versioned hash of the KZG commitment.
pub fn kzg_commitment_to_versioned_hash(kzg_commitment: &KzgCommitment) -> VersionedHash {
    // Hash the contents of the KZG commitment
    let mut hashed_commitment = hash_fixed(&kzg_commitment.0);
    
    // Set the first byte of the hashed commitment to the KZG version constant
    hashed_commitment[0] = VERSIONED_HASH_VERSION_KZG;
    
    // Convert the hashed commitment to a VersionedHash
    VersionedHash::from(hashed_commitment)
}
