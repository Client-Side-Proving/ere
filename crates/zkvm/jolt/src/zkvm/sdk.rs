use crate::{program::JoltProgram, zkvm::Error};
use core::{array::from_fn, cmp::min};
use ere_zkvm_interface::zkvm::PublicValues;
use jolt_common::constants::{
    DEFAULT_HEAP_SIZE, DEFAULT_MAX_INPUT_SIZE, DEFAULT_MAX_OUTPUT_SIZE, DEFAULT_MAX_TRACE_LENGTH,
    DEFAULT_MAX_TRUSTED_ADVICE_SIZE, DEFAULT_MAX_UNTRUSTED_ADVICE_SIZE, DEFAULT_STACK_SIZE,
};
use jolt_sdk::{
    F, JoltDevice, JoltProverPreprocessing, JoltVerifierPreprocessing, MemoryConfig, MemoryLayout,
    RV64IMACProof, TraceInputs, TracerBackend, VerifierPCS, VerifierTranscript, VerifierVC,
    host::JoltProgramSource, preprocess_program, prove_program, verifier_preprocessing_from_prover,
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize)]
pub struct JoltProof {
    proof: RV64IMACProof,
    outputs: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct JoltConfig {
    max_input_size: u64,
    max_trusted_advice_size: u64,
    max_untrusted_advice_size: u64,
    max_output_size: u64,
    stack_size: u64,
    heap_size: u64,
    max_trace_length: u64,
}

impl JoltConfig {
    pub fn from_env() -> Self {
        #[rustfmt::skip]
        let envs = [
            ("JOLT_MAX_INPUT_SIZE",            DEFAULT_MAX_INPUT_SIZE),
            ("JOLT_MAX_TRUSTED_ADVICE_SIZE",   DEFAULT_MAX_TRUSTED_ADVICE_SIZE),
            ("JOLT_MAX_UNTRUSTED_ADVICE_SIZE", DEFAULT_MAX_UNTRUSTED_ADVICE_SIZE),
            ("JOLT_MAX_OUTPUT_SIZE",           DEFAULT_MAX_OUTPUT_SIZE),
            ("JOLT_STACK_SIZE",                DEFAULT_STACK_SIZE),
            ("JOLT_HEAP_SIZE",                 DEFAULT_HEAP_SIZE),
            ("JOLT_MAX_TRACE_LENGTH",          DEFAULT_MAX_TRACE_LENGTH),
        ];
        let [
            max_input_size,
            max_trusted_advice_size,
            max_untrusted_advice_size,
            max_output_size,
            stack_size,
            heap_size,
            max_trace_length,
        ] = envs.map(|(key, default)| {
            env::var(key)
                .ok()
                .and_then(|val| val.parse().ok())
                .unwrap_or(default)
        });
        Self {
            max_input_size,
            max_trusted_advice_size,
            max_untrusted_advice_size,
            max_output_size,
            stack_size,
            heap_size,
            max_trace_length,
        }
    }
}

pub struct JoltSdk {
    program: JoltProgram,
    memory_config: MemoryConfig,
    pk: JoltProverPreprocessing,
    vk: JoltVerifierPreprocessing,
}

impl JoltSdk {
    pub fn new(mut program: JoltProgram, config: JoltConfig) -> Result<Self, Error> {
        let mut memory_config = MemoryConfig {
            max_input_size: config.max_input_size,
            max_trusted_advice_size: config.max_trusted_advice_size,
            max_untrusted_advice_size: config.max_untrusted_advice_size,
            max_output_size: config.max_output_size,
            stack_size: config.stack_size,
            heap_size: config.heap_size,
            program_size: None,
        };
        let pk = preprocess_program(
            &mut program,
            memory_config,
            config.max_trace_length as usize,
            None,
        )
        .map_err(|err| Error::PreprocessingFailed(err.to_string()))?;
        let vk = verifier_preprocessing_from_prover(&pk);
        memory_config.program_size = Some(vk.program.memory_layout().program_size);

        Ok(Self {
            program,
            memory_config,
            pk,
            vk,
        })
    }

    pub fn execute(&self, input: &[u8]) -> Result<(PublicValues, u64), Error> {
        // Untrusted advice carries private inputs; the Jolt device's input stream is public.
        let program = self
            .program
            .build_jolt_program()
            .map_err(|err| Error::ExecutionFailed(err.to_string()))?;
        let trace = program
            .trace_with(
                &mut TracerBackend::new(),
                TraceInputs::new(Vec::new(), input.to_vec(), Vec::new(), self.memory_config),
            )
            .map_err(|err| Error::ExecutionFailed(err.to_string()))?;
        if trace.device.panic {
            return Err(Error::ExecutionPanic);
        }
        let num_cycles = trace.trace.rows().len();
        let public_values = extract_public_values(&trace.device.outputs)?;
        Ok((public_values, num_cycles as _))
    }

    pub fn prove(&self, input: &[u8]) -> Result<(PublicValues, JoltProof), Error> {
        // The SDK's zk feature enables Dory hiding commitments and BlindFold.
        let (proof, io) = prove_program(&self.program, &self.pk, &[], input, &[], None, None, None)
            .map_err(|err| Error::ProveFailed(err.to_string()))?;
        if io.panic {
            return Err(Error::ExecutionPanic);
        }

        let public_values = extract_public_values(&io.outputs)?;
        let proof = JoltProof {
            proof,
            outputs: io.outputs,
        };
        Ok((public_values, proof))
    }

    pub fn verify(&self, proof: JoltProof) -> Result<PublicValues, Error> {
        let io_device = JoltDevice {
            outputs: proof.outputs.clone(),
            panic: false,
            memory_layout: MemoryLayout::new(&self.memory_config),
            ..Default::default()
        };

        jolt_sdk::jolt_verifier::verify::<F, VerifierPCS, VerifierVC, VerifierTranscript>(
            &self.vk,
            &io_device,
            &proof.proof,
            None,
        )
        .map_err(Error::VerifyFailed)?;

        let public_values = extract_public_values(&proof.outputs)?;
        Ok(public_values)
    }
}

// Note that for execute the bytes are padded to size of multiple of 8, but for
// prove the bytes are truncated.
fn extract_public_values(output: &[u8]) -> Result<Vec<u8>, Error> {
    Ok(if output.is_empty() {
        Vec::new()
    } else {
        let len = u32::from_le_bytes(from_fn(|i| output.get(i).copied().unwrap_or(0))) as usize;
        if output.len() > (len + 4).next_multiple_of(8) {
            return Err(Error::InvalidOutput);
        }
        let mut public_values = vec![0; len];
        if let Some((_, output)) = output.split_at_checked(4) {
            let len = min(len, output.len());
            public_values[..len].copy_from_slice(&output[..len]);
        }
        public_values
    })
}
