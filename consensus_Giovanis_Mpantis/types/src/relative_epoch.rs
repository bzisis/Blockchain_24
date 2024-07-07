use crate::*;
use safe_arith::{ArithError, SafeArith};

/// Errors that can occur when working with relative epochs.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    /// The other epoch is too low compared to the base epoch.
    EpochTooLow { base: Epoch, other: Epoch },
    /// The other epoch is too high compared to the base epoch.
    EpochTooHigh { base: Epoch, other: Epoch },
    /// An arithmetic error occurred.
    ArithError(ArithError),
}

impl From<ArithError> for Error {
    fn from(e: ArithError) -> Self {
        Self::ArithError(e)
    }
}

/// Defines the relative positions of epochs with respect to some base epoch.
///
/// This enum is defined according to the specification v0.12.1.
#[derive(Debug, PartialEq, Clone, Copy, arbitrary::Arbitrary)]
pub enum RelativeEpoch {
    /// Refers to the epoch immediately prior to the base epoch.
    Previous,
    /// Refers to the base epoch itself.
    Current,
    /// Refers to the epoch immediately following the base epoch.
    Next,
}

impl RelativeEpoch {
    /// Converts the relative epoch (`self`) into an absolute epoch, relative to the `base` epoch.
    ///
    /// # Returns
    ///
    /// The absolute epoch corresponding to `self`.
    ///
    /// # Specification
    ///
    /// Spec v0.12.1
    pub fn into_epoch(self, base: Epoch) -> Epoch {
        match self {
            RelativeEpoch::Current => base,
            RelativeEpoch::Previous => base.saturating_sub(1u64),
            RelativeEpoch::Next => base.saturating_add(1u64),
        }
    }

    /// Converts an absolute `other` epoch into a `RelativeEpoch`, relative to the `base` epoch.
    ///
    /// # Errors
    ///
    /// - `EpochTooLow`: When `other` is more than 1 epoch prior to `base`.
    /// - `EpochTooHigh`: When `other` is more than 1 epoch after `base`.
    ///
    /// # Specification
    ///
    /// Spec v0.12.1
    pub fn from_epoch(base: Epoch, other: Epoch) -> Result<Self, Error> {
        if other == base {
            Ok(RelativeEpoch::Current)
        } else if other.safe_add(1)? == base {
            Ok(RelativeEpoch::Previous)
        } else if other == base.safe_add(1)? {
            Ok(RelativeEpoch::Next)
        } else if other < base {
            Err(Error::EpochTooLow { base, other })
        } else {
            Err(Error::EpochTooHigh { base, other })
        }
    }

    /// Convenience function to convert slots into relative epochs, based on the `slots_per_epoch`.
    ///
    /// # Errors
    ///
    /// This function internally calls `from_epoch` and propagates any errors from it.
    ///
    /// # Specification
    ///
    /// Spec v0.12.1
    pub fn from_slot(base: Slot, other: Slot, slots_per_epoch: u64) -> Result<Self, Error> {
        Self::from_epoch(base.epoch(slots_per_epoch), other.epoch(slots_per_epoch))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_epoch() {
        let base = Epoch::new(10);

        assert_eq!(RelativeEpoch::Current.into_epoch(base), base);
        assert_eq!(RelativeEpoch::Previous.into_epoch(base), base - 1);
        assert_eq!(RelativeEpoch::Next.into_epoch(base), base + 1);
    }

    #[test]
    fn from_epoch() {
        let base = Epoch::new(10);

        assert_eq!(
            RelativeEpoch::from_epoch(base, base - 1),
            Ok(RelativeEpoch::Previous)
        );
        assert_eq!(
            RelativeEpoch::from_epoch(base, base),
            Ok(RelativeEpoch::Current)
        );
        assert_eq!(
            RelativeEpoch::from_epoch(base, base + 1),
            Ok(RelativeEpoch::Next)
        );
    }

    #[test]
    fn from_slot() {
        let slots_per_epoch: u64 = 64;
        let base = Slot::new(10 * slots_per_epoch);

        assert_eq!(
            RelativeEpoch::from_slot(base, base - 1, slots_per_epoch),
            Ok(RelativeEpoch::Previous)
        );
        assert_eq!(
            RelativeEpoch::from_slot(base, base, slots_per_epoch),
            Ok(RelativeEpoch::Current)
        );
        assert_eq!(
            RelativeEpoch::from_slot(base, base + slots_per_epoch, slots_per_epoch),
            Ok(RelativeEpoch::Next)
        );
    }
}
