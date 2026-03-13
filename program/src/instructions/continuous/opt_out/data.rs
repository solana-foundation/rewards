use pinocchio::error::ProgramError;

use crate::{traits::InstructionData, utils::CONFIDENTIAL_TRANSFER_DATA_LEN};

pub struct ContinuousOptOutData {
    /// Present when the pool has `confidential_rewards` enabled.
    pub confidential_transfer_bytes: Option<[u8; CONFIDENTIAL_TRANSFER_DATA_LEN]>,
}

impl<'a> TryFrom<&'a [u8]> for ContinuousOptOutData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        let confidential_transfer_bytes = if data.len() >= CONFIDENTIAL_TRANSFER_DATA_LEN {
            Some(data[..CONFIDENTIAL_TRANSFER_DATA_LEN].try_into().map_err(|_| ProgramError::InvalidInstructionData)?)
        } else {
            None
        };

        Ok(Self { confidential_transfer_bytes })
    }
}

impl<'a> InstructionData<'a> for ContinuousOptOutData {
    const LEN: usize = 0;
}
