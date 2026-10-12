use ere_zkvm_interface::zkvm::CommonError;
use jolt_sdk::jolt_verifier::VerifierError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    CommonError(#[from] CommonError),

    #[error("Output is expected to have length prefix")]
    InvalidOutput,

    // Execute
    #[error("Execution panics")]
    ExecutionPanic,

    #[error("Jolt preprocessing failed: {0}")]
    PreprocessingFailed(String),

    #[error("Jolt execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Jolt proving failed: {0}")]
    ProveFailed(String),

    // Verify
    #[error("Verification failed: {0}")]
    VerifyFailed(#[from] VerifierError),
}
