//! Implementations of SQLite compatibility traits.

use crate::{Epoch, Slot};
use rusqlite::{
    types::{FromSql, FromSqlError, ToSql, ToSqlOutput, ValueRef},
    Error,
};

/// Implements `ToSql` and `FromSql` for a given type, enabling compatibility with SQLite.
macro_rules! impl_to_from_sql {
    ($type:ty) => {
        /// Implements `ToSql` for `$type`, converting it into an SQLite-compatible representation.
        impl ToSql for $type {
            /// Converts `$type` to SQLite representation.
            fn to_sql(&self) -> Result<ToSqlOutput, Error> {
                // Convert `$type` to `i64` and then to `ToSqlOutput`.
                let val_i64 = i64::try_from(self.as_u64())
                    .map_err(|e| Error::ToSqlConversionFailure(Box::new(e)))?;
                Ok(ToSqlOutput::from(val_i64))
            }
        }

        /// Implements `FromSql` for `$type`, converting SQLite value to `$type`.
        impl FromSql for $type {
            /// Converts SQLite `ValueRef` to `$type`.
            fn column_result(value: ValueRef) -> Result<Self, FromSqlError> {
                // Convert SQLite `ValueRef` to `i64` and then to `$type`.
                let val_u64 = u64::try_from(i64::column_result(value)?)
                    .map_err(|e| FromSqlError::Other(Box::new(e)))?;
                Ok(Self::new(val_u64))
            }
        }
    };
}

// Implement `ToSql` and `FromSql` for `Slot` and `Epoch`.
impl_to_from_sql!(Slot);
impl_to_from_sql!(Epoch);
