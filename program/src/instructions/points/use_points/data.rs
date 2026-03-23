use pinocchio::error::ProgramError;

use crate::{errors::RewardsProgramError, require_len, traits::InstructionData};

pub struct UsePointsData {
    pub quantity: u64,
}

impl<'a> TryFrom<&'a [u8]> for UsePointsData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let quantity = u64::from_le_bytes(data[0..8].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        Ok(Self { quantity })
    }
}

impl<'a> InstructionData<'a> for UsePointsData {
    const LEN: usize = 8; // quantity(8)

    fn validate(&self) -> Result<(), ProgramError> {
        if self.quantity == 0 {
            return Err(RewardsProgramError::InvalidAmount.into());
        }
        Ok(())
    }
}
