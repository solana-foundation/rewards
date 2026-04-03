use pinocchio::error::ProgramError;

use crate::{errors::RewardsProgramError, require_len, traits::InstructionData};

pub struct DistributeContinuousRewardData {
    pub amount: u64,
}

impl<'a> TryFrom<&'a [u8]> for DistributeContinuousRewardData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let amount = u64::from_le_bytes(data[0..8].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        Ok(Self { amount })
    }
}

impl<'a> InstructionData<'a> for DistributeContinuousRewardData {
    const LEN: usize = 8;

    fn validate(&self) -> Result<(), ProgramError> {
        if self.amount == 0 {
            return Err(RewardsProgramError::InvalidAmount.into());
        }
        Ok(())
    }
}
