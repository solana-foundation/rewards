use pinocchio::error::ProgramError;

use crate::{require_len, traits::InstructionData};

pub struct ClaimContinuousData {
    pub amount: u64,
}

impl<'a> TryFrom<&'a [u8]> for ClaimContinuousData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let amount = u64::from_le_bytes(data[0..8].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        Ok(Self { amount })
    }
}

impl<'a> InstructionData<'a> for ClaimContinuousData {
    const LEN: usize = 8;
}
