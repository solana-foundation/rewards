use pinocchio::{account::AccountView, error::ProgramError};

/// Discriminators for the Rewards Program instructions.
#[repr(u8)]
pub enum RewardsInstructionDiscriminators {
    // Direct Distribution
    CreateDirectDistribution = 0,
    AddDirectRecipient = 1,
    ClaimDirect = 2,
    CloseDirectDistribution = 3,
    CloseDirectRecipient = 4,

    // Merkle Distribution
    CreateMerkleDistribution = 5,
    ClaimMerkle = 6,
    CloseMerkleClaim = 7,
    CloseMerkleDistribution = 8,

    // Revoke
    RevokeDirectRecipient = 9,
    RevokeMerkleClaim = 10,

    // Continuous Distribution
    CreateContinuousPool = 11,
    ContinuousOptIn = 12,
    ContinuousOptOut = 13,
    DistributeContinuousReward = 14,
    ClaimContinuous = 15,
    SyncContinuousBalance = 16,
    SetContinuousBalance = 17,
    CloseContinuousPool = 18,
    RevokeContinuousUser = 19,

    // Shared
    EmitEvent = 228,
}

impl TryFrom<u8> for RewardsInstructionDiscriminators {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            // Direct Distribution
            0 => Ok(Self::CreateDirectDistribution),
            1 => Ok(Self::AddDirectRecipient),
            2 => Ok(Self::ClaimDirect),
            3 => Ok(Self::CloseDirectDistribution),
            4 => Ok(Self::CloseDirectRecipient),
            // Merkle Distribution
            5 => Ok(Self::CreateMerkleDistribution),
            6 => Ok(Self::ClaimMerkle),
            7 => Ok(Self::CloseMerkleClaim),
            8 => Ok(Self::CloseMerkleDistribution),
            // Revoke
            9 => Ok(Self::RevokeDirectRecipient),
            10 => Ok(Self::RevokeMerkleClaim),
            // Continuous Distribution
            11 => Ok(Self::CreateContinuousPool),
            12 => Ok(Self::ContinuousOptIn),
            13 => Ok(Self::ContinuousOptOut),
            14 => Ok(Self::DistributeContinuousReward),
            15 => Ok(Self::ClaimContinuous),
            16 => Ok(Self::SyncContinuousBalance),
            17 => Ok(Self::SetContinuousBalance),
            18 => Ok(Self::CloseContinuousPool),
            19 => Ok(Self::RevokeContinuousUser),
            // Shared
            228 => Ok(Self::EmitEvent),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

/// Marker trait for instruction account structs
///
/// Implementors should use TryFrom<&'a [AccountView]> for parsing
pub trait InstructionAccounts<'a>: Sized + TryFrom<&'a [AccountView], Error = ProgramError> {}

/// Marker trait for instruction data structs
///
/// Implementors should use TryFrom<&'a [u8]> for parsing
pub trait InstructionData<'a>: Sized + TryFrom<&'a [u8], Error = ProgramError> {
    /// Expected length of instruction data
    const LEN: usize;

    /// Validates the instruction data. Override for custom validation.
    fn validate(&self) -> Result<(), ProgramError> {
        Ok(())
    }
}

/// Full instruction combining accounts and data
///
/// Implementors get automatic TryFrom<(&'a [u8], &'a [AccountView])>
pub trait Instruction<'a>: Sized {
    type Accounts: InstructionAccounts<'a>;
    type Data: InstructionData<'a>;

    fn accounts(&self) -> &Self::Accounts;
    fn data(&self) -> &Self::Data;

    /// Parse instruction from data and accounts tuple
    #[inline(always)]
    fn parse(data: &'a [u8], accounts: &'a [AccountView]) -> Result<Self, ProgramError>
    where
        Self: From<(Self::Accounts, Self::Data)>,
    {
        let accounts = Self::Accounts::try_from(accounts)?;
        let data = Self::Data::try_from(data)?;
        Ok(Self::from((accounts, data)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discriminator_try_from_create_direct_distribution() {
        let result = RewardsInstructionDiscriminators::try_from(0u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::CreateDirectDistribution));
    }

    #[test]
    fn test_discriminator_try_from_add_direct_recipient() {
        let result = RewardsInstructionDiscriminators::try_from(1u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::AddDirectRecipient));
    }

    #[test]
    fn test_discriminator_try_from_claim_direct() {
        let result = RewardsInstructionDiscriminators::try_from(2u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::ClaimDirect));
    }

    #[test]
    fn test_discriminator_try_from_close_direct_distribution() {
        let result = RewardsInstructionDiscriminators::try_from(3u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::CloseDirectDistribution));
    }

    #[test]
    fn test_discriminator_try_from_emit_event() {
        let result = RewardsInstructionDiscriminators::try_from(228u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::EmitEvent));
    }

    #[test]
    fn test_discriminator_try_from_merkle_instructions() {
        let result = RewardsInstructionDiscriminators::try_from(5u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::CreateMerkleDistribution));

        let result = RewardsInstructionDiscriminators::try_from(6u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::ClaimMerkle));

        let result = RewardsInstructionDiscriminators::try_from(7u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::CloseMerkleClaim));

        let result = RewardsInstructionDiscriminators::try_from(8u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::CloseMerkleDistribution));
    }

    #[test]
    fn test_discriminator_try_from_close_direct_recipient() {
        let result = RewardsInstructionDiscriminators::try_from(4u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::CloseDirectRecipient));
    }

    #[test]
    fn test_discriminator_try_from_revoke_direct_recipient() {
        let result = RewardsInstructionDiscriminators::try_from(9u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::RevokeDirectRecipient));
    }

    #[test]
    fn test_discriminator_try_from_revoke_merkle_claim() {
        let result = RewardsInstructionDiscriminators::try_from(10u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::RevokeMerkleClaim));
    }

    #[test]
    fn test_discriminator_try_from_continuous_instructions() {
        let result = RewardsInstructionDiscriminators::try_from(11u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::CreateContinuousPool));

        let result = RewardsInstructionDiscriminators::try_from(12u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::ContinuousOptIn));

        let result = RewardsInstructionDiscriminators::try_from(13u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::ContinuousOptOut));

        let result = RewardsInstructionDiscriminators::try_from(14u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::DistributeContinuousReward));

        let result = RewardsInstructionDiscriminators::try_from(15u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::ClaimContinuous));

        let result = RewardsInstructionDiscriminators::try_from(16u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::SyncContinuousBalance));

        let result = RewardsInstructionDiscriminators::try_from(17u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::SetContinuousBalance));

        let result = RewardsInstructionDiscriminators::try_from(18u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::CloseContinuousPool));

        let result = RewardsInstructionDiscriminators::try_from(19u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::RevokeContinuousUser));
    }

    #[test]
    fn test_discriminator_try_from_invalid() {
        let result = RewardsInstructionDiscriminators::try_from(20u8);
        assert!(matches!(result, Err(ProgramError::InvalidInstructionData)));

        let result = RewardsInstructionDiscriminators::try_from(255u8);
        assert!(matches!(result, Err(ProgramError::InvalidInstructionData)));

        let result = RewardsInstructionDiscriminators::try_from(100u8);
        assert!(matches!(result, Err(ProgramError::InvalidInstructionData)));
    }
}
