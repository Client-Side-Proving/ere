use serde::{Deserialize, Serialize};

/// Jolt program that contains ELF of compiled guest.
#[derive(Clone, Serialize, Deserialize)]
pub struct JoltProgram {
    pub(crate) elf: Vec<u8>,
}

impl JoltProgram {
    pub fn elf(&self) -> &[u8] {
        &self.elf
    }
}

#[cfg(any(feature = "compiler", feature = "zkvm"))]
impl jolt_sdk::host::JoltProgramSource for JoltProgram {
    fn get_elf_contents(&self) -> Option<Vec<u8>> {
        Some(self.elf.clone())
    }

    fn get_elf_compute_advice_contents(&self) -> Option<Vec<u8>> {
        None
    }
}
