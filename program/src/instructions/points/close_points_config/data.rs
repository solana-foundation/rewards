use pinocchio::error::ProgramError;

use crate::traits::InstructionData;

pub struct ClosePointsConfigData;

impl<'a> TryFrom<&'a [u8]> for ClosePointsConfigData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(_data: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl<'a> InstructionData<'a> for ClosePointsConfigData {
    const LEN: usize = 0;
}
