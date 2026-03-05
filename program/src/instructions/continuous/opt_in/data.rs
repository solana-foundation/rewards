use pinocchio::error::ProgramError;

use crate::{require_len, traits::InstructionData};

pub struct ContinuousOptInData {
    pub bump: u8,
}

impl<'a> TryFrom<&'a [u8]> for ContinuousOptInData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);
        Ok(Self { bump: data[0] })
    }
}

impl<'a> InstructionData<'a> for ContinuousOptInData {
    const LEN: usize = 1;
}
