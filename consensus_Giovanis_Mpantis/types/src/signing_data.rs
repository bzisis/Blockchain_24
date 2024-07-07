use crate::test_utils::TestRandom;
use crate::Hash256;

use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash::TreeHash;
use tree_hash_derive::TreeHash;

/// A struct representing signing data, which includes an object root and a domain.
/// 
/// # Fields
/// 
/// * `object_root` - A `Hash256` representing the object root.
/// * `domain` - A `Hash256` representing the domain.
/// 
/// # Derives
/// 
/// This struct derives the following traits:
/// 
/// * `arbitrary::Arbitrary`
/// * `Debug`
/// * `PartialEq`
/// * `Clone`
/// * `Serialize`
/// * `Deserialize`
/// * `Encode`
/// * `Decode`
/// * `TreeHash`
/// * `TestRandom`
#[derive(
    arbitrary::Arbitrary,
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
)]
pub struct SigningData {
    pub object_root: Hash256,
    pub domain: Hash256,
}

/// A trait for types that have a signed root.
/// 
/// This trait extends the `TreeHash` trait and provides a method for calculating the signing root.
/// 
/// # Provided methods
/// 
/// * `signing_root` - Calculates the signing root by creating a `SigningData` instance with the
///   `object_root` set to the result of `tree_hash_root()` and the provided `domain`, and then
///   calling `tree_hash_root()` on this `SigningData` instance.
/// 
/// # Example
/// 
/// ```rust
/// struct MyStruct {
///     // Your fields here
/// }
/// 
/// impl TreeHash for MyStruct {
///     fn tree_hash_root(&self) -> Hash256 {
///         // Your implementation here
///     }
/// }
/// 
/// impl SignedRoot for MyStruct {}
/// 
/// let my_struct = MyStruct {
///     // Initialize your fields here
/// };
/// 
/// let domain = Hash256::zero(); // Or any other appropriate value
/// let root = my_struct.signing_root(domain);
/// ```
pub trait SignedRoot: TreeHash {
    fn signing_root(&self, domain: Hash256) -> Hash256 {
        SigningData {
            object_root: self.tree_hash_root(),
            domain,
        }
        .tree_hash_root()
    }
}
