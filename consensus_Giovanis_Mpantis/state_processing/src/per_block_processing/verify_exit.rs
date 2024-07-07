use super::errors::{BlockOperationError, ExitInvalid};
use crate::per_block_processing::{
    signature_sets::{exit_signature_set, get_pubkey_from_state},
    VerifySignatures,
};
use safe_arith::SafeArith;
use types::*;

type Result<T> = std::result::Result<T, BlockOperationError<ExitInvalid>>;

/// Creates a `BlockOperationError` with the provided `ExitInvalid` reason.
fn error(reason: ExitInvalid) -> BlockOperationError<ExitInvalid> {
    BlockOperationError::invalid(reason)
}

/// Indicates if an `Exit` is valid to be included in a block in the current epoch of the given
/// state.
///
/// Returns `Ok(())` if the `Exit` is valid, otherwise indicates the reason for invalidity.
///
/// Spec v0.12.1
///
/// # Arguments
///
/// * `state` - The beacon state where the exit is being verified.
/// * `current_epoch` - Optional current epoch; if `None`, `state.current_epoch()` is used.
/// * `signed_exit` - The signed voluntary exit to be verified.
/// * `verify_signatures` - Enum specifying whether to verify signatures.
/// * `spec` - The chain specification defining protocol parameters.
///
/// # Errors
///
/// Returns a `BlockOperationError` indicating the specific reason for the exit's invalidity.
pub fn verify_exit<E: EthSpec>(
    state: &BeaconState<E>,
    current_epoch: Option<Epoch>,
    signed_exit: &SignedVoluntaryExit,
    verify_signatures: VerifySignatures,
    spec: &ChainSpec,
) -> Result<()> {
    let current_epoch = current_epoch.unwrap_or(state.current_epoch());
    let exit = &signed_exit.message;

    let validator = state
        .validators()
        .get(exit.validator_index as usize)
        .ok_or_else(|| error(ExitInvalid::ValidatorUnknown(exit.validator_index)))?;

    // Verify the validator is active.
    verify!(
        validator.is_active_at(current_epoch),
        ExitInvalid::NotActive(exit.validator_index)
    );

    // Verify that the validator has not yet exited.
    verify!(
        validator.exit_epoch == spec.far_future_epoch,
        ExitInvalid::AlreadyExited(exit.validator_index)
    );

    // Exits must specify an epoch when they become valid; they are not valid before then.
    verify!(
        current_epoch >= exit.epoch,
        ExitInvalid::FutureEpoch {
            state: current_epoch,
            exit: exit.epoch
        }
    );

    // Verify the validator has been active long enough.
    let earliest_exit_epoch = validator
        .activation_epoch
        .safe_add(spec.shard_committee_period)?;
    verify!(
        current_epoch >= earliest_exit_epoch,
        ExitInvalid::TooYoungToExit {
            current_epoch,
            earliest_exit_epoch,
        }
    );

    // Optionally verify the exit's signatures if specified.
    if verify_signatures.is_true() {
        verify!(
            exit_signature_set(
                state,
                |i| get_pubkey_from_state(state, i),
                signed_exit,
                spec
            )?
            .verify(),
            ExitInvalid::BadSignature
        );
    }

    Ok(())
}
