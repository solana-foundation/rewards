use pinocchio::error::ProgramError;

use crate::{require_len, traits::InstructionData};

pub struct InitPointsData {
    pub bump: u8,
    pub transferable: u8,
    pub revocable: u8,
    pub mint_bump: u8,
}

impl<'a> TryFrom<&'a [u8]> for InitPointsData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let bump = data[0];
        let transferable = data[1];
        let revocable = data[2];
        let mint_bump = data[3];

        Ok(Self { bump, transferable, revocable, mint_bump })
    }
}

impl<'a> InstructionData<'a> for InitPointsData {
    const LEN: usize = 4; // bump(1) + transferable(1) + revocable(1) + mint_bump(1)
}
