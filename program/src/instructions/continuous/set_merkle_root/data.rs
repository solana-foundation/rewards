use pinocchio::error::ProgramError;

use crate::{errors::RewardsProgramError, require_len, traits::InstructionData};

pub struct SetContinuousMerkleRootData {
    pub merkle_root: [u8; 32],
    pub root_version: u64,
}

impl<'a> TryFrom<&'a [u8]> for SetContinuousMerkleRootData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let merkle_root: [u8; 32] = data[0..32].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
        let root_version =
            u64::from_le_bytes(data[32..40].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        Ok(Self { merkle_root, root_version })
    }
}

impl<'a> InstructionData<'a> for SetContinuousMerkleRootData {
    const LEN: usize = 40; // merkle_root(32) + root_version(8)

    fn validate(&self) -> Result<(), ProgramError> {
        if self.root_version == 0 {
            return Err(RewardsProgramError::InvalidMerkleRootVersion.into());
        }
        Ok(())
    }
}
