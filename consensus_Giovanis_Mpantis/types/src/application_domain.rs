/// This value represents an application index of 0 with the bitmask applied (so it's equivalent to the bitmask).
/// 
/// Little endian hex: `0x00000001`
/// Binary: `1000000000000000000000000`
pub const APPLICATION_DOMAIN_BUILDER: u32 = 16777216;

/// Enum representing different application domains.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ApplicationDomain {
    /// The Builder application domain.
    Builder,
}

impl ApplicationDomain {
    /// Returns the domain constant associated with the enum variant.
    ///
    /// # Returns
    ///
    /// - `APPLICATION_DOMAIN_BUILDER` if `self` is `ApplicationDomain::Builder`.
    pub fn get_domain_constant(&self) -> u32 {
        match self {
            ApplicationDomain::Builder => APPLICATION_DOMAIN_BUILDER,
        }
    }
}
