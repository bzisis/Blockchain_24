/// This module contains functions related to upgrading to Altair.
pub mod altair;

/// This module contains functions related to upgrading to Bellatrix.
pub mod bellatrix;

/// This module contains functions related to upgrading to Capella.
pub mod capella;

/// This module contains functions related to upgrading to Deneb.
pub mod deneb;

/// This module contains functions related to upgrading to Electra.
pub mod electra;

/// Re-export of the function `upgrade_to_altair` from the `altair` module.
pub use altair::upgrade_to_altair;

/// Re-export of the function `upgrade_to_bellatrix` from the `bellatrix` module.
pub use bellatrix::upgrade_to_bellatrix;

/// Re-export of the function `upgrade_to_capella` from the `capella` module.
pub use capella::upgrade_to_capella;

/// Re-export of the function `upgrade_to_deneb` from the `deneb` module.
pub use deneb::upgrade_to_deneb;

/// Re-export of the function `upgrade_to_electra` from the `electra` module.
pub use electra::upgrade_to_electra;
