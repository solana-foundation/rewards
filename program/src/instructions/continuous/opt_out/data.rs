use pinocchio::error::ProgramError;

use crate::traits::InstructionData;

pub struct ContinuousOptOutData;

impl<'a> TryFrom<&'a [u8]> for ContinuousOptOutData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(_data: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl<'a> InstructionData<'a> for ContinuousOptOutData {
    const LEN: usize = 0;
}
