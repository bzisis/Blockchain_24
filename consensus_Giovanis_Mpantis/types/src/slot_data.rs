use crate::Slot;

/// A trait providing a `Slot` getter for messages that are related to a single slot. 
/// This trait is useful in making parts of attestation and sync committee processing generic.
pub trait SlotData {
    /// Returns the slot associated with the message.
    ///
    /// # Returns
    /// 
    /// * `Slot` - The slot associated with the message.
    fn get_slot(&self) -> Slot;
}

impl SlotData for Slot {
    /// Implements the `get_slot` method for the `Slot` type.
    ///
    /// # Returns
    /// 
    /// * `Slot` - The slot itself.
    fn get_slot(&self) -> Slot {
        *self
    }
}
