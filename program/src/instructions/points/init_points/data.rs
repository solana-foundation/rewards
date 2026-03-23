use pinocchio::error::ProgramError;

use crate::{require_len, traits::InstructionData};

pub struct InitPointsData {
    pub bump: u8,
    pub transferable: u8,
    pub revocable: u8,
    pub max_supply: u64,
}

impl<'a> TryFrom<&'a [u8]> for InitPointsData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let bump = data[0];
        let transferable = data[1];
        let revocable = data[2];
        let max_supply = u64::from_le_bytes(data[3..11].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        Ok(Self { bump, transferable, revocable, max_supply })
    }
}

impl<'a> InstructionData<'a> for InitPointsData {
    const LEN: usize = 11; // bump(1) + transferable(1) + revocable(1) + max_supply(8)
}
