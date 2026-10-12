//! Risc0 [`Compiler`] and [`zkVM`] implementation.
//!
//! # Requirements
//!
//! To install all requirements, run `scripts/sdk_installers/install_risc0_sdk.sh` from the Ere
//! repository at the same git revision as your `ere-risc0` dependency.
//!
//! To install `r0vm-cuda` (with GPU proving support), make sure CUDA 12.9 is
//! installed, run `scripts/sdk_installers/install_risc0_sdk.sh` with env `CUDA=1` set.
//!
//! ## `Compiler` requirements
//!
//! - [`rzup`]
//! - `rzup install rust 1.97.0` - Custom Rust toolchain used by `RustRv32imaCustomized`
//!
//! ## `zkVM` requirements
//!
//! - [`rzup`]
//! - `rzup install r0vm 3.0.6`
//! - `r0vm-cuda` - Used by `zkVM::prove` if `ProverResource::Gpu` is
//!   selected
//! - `docker` - Used by `zkVM::prove` if `ProofKind::Groth16` is selected
//!
//! # `Compiler` implementation
//!
//! ## Available compilers
//!
//! | Compiler                | Language | Target                        | Note               |
//! | ----------------------- | :------: | ----------------------------- | ------------------ |
//! | `RustRv32imaCustomized` |   Rust   | `riscv32im-risc0-zkvm-elf`    | With `std` support |
//! | `RustRv32ima`           |   Rust   | `riscv32ima-unknown-none-elf` |                    |
//!
//! # `zkVM` implementation
//!
//! ## Supported `ProverResource`
//!
//! | Resource  | Supported |
//! | --------- | :-------: |
//! | `Cpu`     |    Yes    |
//! | `Gpu`     |    Yes    |
//! | `Network` |    No     |
//! | `Cluster` |    No     |
//!
//! [`rzup`]: https://risczero.com/install

#![cfg_attr(
    all(not(test), feature = "compiler", feature = "zkvm"),
    warn(unused_crate_dependencies)
)]

pub mod program;

#[cfg(feature = "compiler")]
pub mod compiler;

#[cfg(feature = "zkvm")]
pub mod zkvm;

#[cfg(feature = "zkvm")]
pub use zkvm::*;
