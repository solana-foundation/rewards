use pinocchio::error::ProgramError;

use crate::traits::InstructionData;

pub struct ClosePointsAccountData;

impl<'a> TryFrom<&'a [u8]> for ClosePointsAccountData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(_data: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl<'a> InstructionData<'a> for ClosePointsAccountData {
    const LEN: usize = 0;
}
