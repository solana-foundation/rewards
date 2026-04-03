use pinocchio::error::ProgramError;

use crate::traits::InstructionData;

pub struct CloseContinuousPoolData;

impl<'a> TryFrom<&'a [u8]> for CloseContinuousPoolData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(_data: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl<'a> InstructionData<'a> for CloseContinuousPoolData {
    const LEN: usize = 0;
}
