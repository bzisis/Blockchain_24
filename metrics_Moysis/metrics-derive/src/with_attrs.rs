// Moysis Moysis Volos, Greece 29/06/2024.

use syn::{Attribute, DeriveInput, Field};

/// Trait to provide access to attributes for different types.
///
/// This trait defines a method to retrieve a slice of attributes associated with an instance.
/// It is implemented for types that contain attributes, such as `DeriveInput` and `Field`.
pub(crate) trait WithAttrs {
    /// Returns a slice of attributes associated with the instance.
    fn attrs(&self) -> &[Attribute];
}

impl WithAttrs for DeriveInput {
    /// Returns a slice of attributes associated with the `DeriveInput`.
    ///
    /// This implementation allows `DeriveInput` to access its attributes using the `WithAttrs` trait.
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
}

impl WithAttrs for Field {
    /// Returns a slice of attributes associated with the `Field`.
    ///
    /// This implementation allows `Field` to access its attributes using the `WithAttrs` trait.
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
}