use pinocchio::error::ProgramError;

use crate::traits::InstructionData;

pub struct RevokePointsData;

impl<'a> TryFrom<&'a [u8]> for RevokePointsData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(_data: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl<'a> InstructionData<'a> for RevokePointsData {
    const LEN: usize = 0;
}
