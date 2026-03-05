use alloc::vec::Vec;
use pinocchio::{account::AccountView, error::ProgramError, Address};

use crate::{require_len, validate_discriminator};

pub const ACCOUNT_DISCRIMINATOR_SIZE: usize = 1;
pub const ACCOUNT_VERSION_SIZE: usize = 1;
pub const ACCOUNT_HEADER_SIZE: usize = ACCOUNT_DISCRIMINATOR_SIZE + ACCOUNT_VERSION_SIZE;

/// Discriminator for account types
pub trait Discriminator {
    const DISCRIMINATOR: u8;
}

/// Version marker for account types
pub trait Versioned {
    const VERSION: u8;
}

/// Account size constants
pub trait AccountSize: Discriminator + Versioned + Sized {
    /// Size of the account data (excluding discriminator and version)
    const DATA_LEN: usize;

    /// Total size including discriminator and version
    const LEN: usize = 1 + 1 + Self::DATA_LEN;

    /// Alias for LEN for backwards compatibility
    const ACCOUNT_SIZE: usize = Self::LEN;
}

/// Zero-copy account deserialization
pub trait AccountDeserialize: AccountSize {
    /// Zero-copy read from byte slice (validates discriminator, skips version)
    #[inline(always)]
    fn from_bytes(data: &[u8]) -> Result<&Self, ProgramError> {
        require_len!(data, Self::LEN);
        validate_discriminator!(data, Self::DISCRIMINATOR);

        // Skip discriminator (byte 0) and version (byte 1)
        unsafe { Self::from_bytes_unchecked(&data[2..]) }
    }

    /// Zero-copy read without discriminator validation
    ///
    /// # Safety
    /// Caller must ensure data is valid, properly sized, and aligned.
    /// Struct must be `#[repr(C)]` with no padding.
    #[inline(always)]
    unsafe fn from_bytes_unchecked(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(&*(data.as_ptr() as *const Self))
    }

    /// Mutable zero-copy access
    #[inline(always)]
    fn from_bytes_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        require_len!(data, Self::LEN);
        validate_discriminator!(data, Self::DISCRIMINATOR);

        // Skip discriminator (byte 0) and version (byte 1)
        unsafe { Self::from_bytes_mut_unchecked(&mut data[2..]) }
    }

    /// Mutable zero-copy access without validation
    ///
    /// # Safety
    /// Caller must ensure data is valid, properly sized, and aligned.
    /// Struct must be `#[repr(C)]` with no padding.
    #[inline(always)]
    unsafe fn from_bytes_mut_unchecked(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(&mut *(data.as_mut_ptr() as *mut Self))
    }
}

/// Account discriminator values for this program
#[repr(u8)]
pub enum RewardsAccountDiscriminators {
    DirectDistribution = 0,
    DirectRecipient = 1,
    MerkleDistribution = 2,
    MerkleClaim = 3,
    Revocation = 4,
    RewardPool = 5,
    UserRewardAccount = 6,
}

/// Manual account deserialization (non-zero-copy)
///
/// Use this for accounts where zero-copy deserialization isn't possible
/// due to alignment constraints.
pub trait AccountParse: AccountSize {
    /// Parse account from bytes (validates discriminator, skips version)
    fn parse_from_bytes(data: &[u8]) -> Result<Self, ProgramError>;
}

/// Account serialization with discriminator and version prefix
pub trait AccountSerialize: Discriminator + Versioned {
    /// Serialize account data without discriminator/version
    fn to_bytes_inner(&self) -> Vec<u8>;

    /// Serialize with discriminator and version prefix
    #[inline(always)]
    fn to_bytes(&self) -> Vec<u8> {
        let inner = self.to_bytes_inner();
        let mut data = Vec::with_capacity(1 + 1 + inner.len());
        data.push(Self::DISCRIMINATOR);
        data.push(Self::VERSION);
        data.extend_from_slice(&inner);
        data
    }

    /// Write directly to a mutable slice
    #[inline(always)]
    fn write_to_slice(&self, dest: &mut [u8]) -> Result<(), ProgramError> {
        let bytes = self.to_bytes();
        if dest.len() < bytes.len() {
            return Err(ProgramError::AccountDataTooSmall);
        }
        dest[..bytes.len()].copy_from_slice(&bytes);
        Ok(())
    }
}

/// Account validation helpers
pub trait AccountValidation: Discriminator {
    fn validate_discriminator(data: &[u8]) -> Result<(), ProgramError> {
        if data.is_empty() || data[0] != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    fn validate_owner(account: &AccountView, expected_owner: &Address) -> Result<(), ProgramError> {
        if !account.owned_by(expected_owner) {
            return Err(ProgramError::IllegalOwner);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use super::*;
    use alloc::vec;

    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct TestAccount {
        pub bump: u8,
        pub data: [u8; 32],
    }

    impl Discriminator for TestAccount {
        const DISCRIMINATOR: u8 = 0;
    }

    impl Versioned for TestAccount {
        const VERSION: u8 = 1;
    }

    impl AccountSize for TestAccount {
        const DATA_LEN: usize = 1 + 32;
    }

    impl AccountDeserialize for TestAccount {}

    impl AccountSerialize for TestAccount {
        fn to_bytes_inner(&self) -> Vec<u8> {
            let mut bytes = vec![self.bump];
            bytes.extend_from_slice(&self.data);
            bytes
        }
    }

    #[test]
    fn test_from_bytes_mut_modifies_original() {
        let account = TestAccount { bump: 100, data: [1u8; 32] };
        let mut bytes = account.to_bytes();

        {
            let account_mut = TestAccount::from_bytes_mut(&mut bytes).unwrap();
            account_mut.bump = 200;
        }

        let deserialized = TestAccount::from_bytes(&bytes).unwrap();
        assert_eq!(deserialized.bump, 200);
    }

    #[test]
    fn test_from_bytes_unchecked_skips_discriminator_and_version() {
        let account = TestAccount { bump: 100, data: [1u8; 32] };
        let bytes = account.to_bytes();

        // Skip discriminator (byte 0) and version (byte 1)
        let result = unsafe { TestAccount::from_bytes_unchecked(&bytes[2..]) };
        assert!(result.is_ok());

        let deserialized = result.unwrap();
        assert_eq!(deserialized.bump, 100);
    }

    #[test]
    fn test_from_bytes_unchecked_too_short() {
        let data = [0u8; 4];
        let result = unsafe { TestAccount::from_bytes_unchecked(&data) };
        assert_eq!(result, Err(ProgramError::InvalidAccountData));
    }

    #[test]
    fn test_to_bytes_roundtrip() {
        let account = TestAccount { bump: 123, data: [5u8; 32] };

        let bytes = account.to_bytes();
        let deserialized = TestAccount::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.bump, account.bump);
        assert_eq!(deserialized.data, account.data);
    }

    #[test]
    fn test_write_to_slice_exact_size() {
        let account = TestAccount { bump: 100, data: [1u8; 32] };

        let mut dest = vec![0u8; TestAccount::LEN];
        assert!(account.write_to_slice(&mut dest).is_ok());

        let deserialized = TestAccount::from_bytes(&dest).unwrap();
        assert_eq!(deserialized.bump, 100);
    }

    #[test]
    fn test_version_auto_serialized() {
        let account = TestAccount { bump: 100, data: [1u8; 32] };

        let bytes = account.to_bytes();

        // Byte 0 = discriminator, Byte 1 = version
        assert_eq!(bytes[0], TestAccount::DISCRIMINATOR);
        assert_eq!(bytes[1], TestAccount::VERSION);
    }

    #[test]
    fn test_from_bytes_invalid_discriminator() {
        let account = TestAccount { bump: 100, data: [1u8; 32] };
        let mut bytes = account.to_bytes();
        bytes[0] = 99; // wrong discriminator
        let result = TestAccount::from_bytes(&bytes);
        assert_eq!(result, Err(ProgramError::InvalidAccountData));
    }

    #[test]
    fn test_from_bytes_too_short() {
        let bytes = [0u8; 5];
        let result = TestAccount::from_bytes(&bytes);
        assert_eq!(result, Err(ProgramError::InvalidInstructionData));
    }

    #[test]
    fn test_from_bytes_mut_too_short() {
        let mut bytes = [0u8; 5];
        let result = TestAccount::from_bytes_mut(&mut bytes);
        assert_eq!(result, Err(ProgramError::InvalidInstructionData));
    }

    #[test]
    fn test_from_bytes_mut_invalid_discriminator() {
        let account = TestAccount { bump: 100, data: [1u8; 32] };
        let mut bytes = account.to_bytes();
        bytes[0] = 99; // wrong discriminator
        let result = TestAccount::from_bytes_mut(&mut bytes);
        assert_eq!(result, Err(ProgramError::InvalidAccountData));
    }

    #[test]
    fn test_write_to_slice_too_small() {
        let account = TestAccount { bump: 100, data: [1u8; 32] };
        let mut dest = vec![0u8; 5];
        let result = account.write_to_slice(&mut dest);
        assert_eq!(result, Err(ProgramError::AccountDataTooSmall));
    }

    #[test]
    fn test_write_to_slice_larger_buffer() {
        let account = TestAccount { bump: 100, data: [1u8; 32] };
        let mut dest = vec![0u8; TestAccount::LEN + 50];
        assert!(account.write_to_slice(&mut dest).is_ok());

        let deserialized = TestAccount::from_bytes(&dest).unwrap();
        assert_eq!(deserialized.bump, 100);
    }

    impl AccountValidation for TestAccount {}

    #[test]
    fn test_validate_discriminator_success() {
        let account = TestAccount { bump: 100, data: [1u8; 32] };
        let bytes = account.to_bytes();
        assert!(TestAccount::validate_discriminator(&bytes).is_ok());
    }

    #[test]
    fn test_validate_discriminator_wrong() {
        let bytes = [99u8; 10];
        assert_eq!(TestAccount::validate_discriminator(&bytes), Err(ProgramError::InvalidAccountData));
    }

    #[test]
    fn test_validate_discriminator_empty() {
        let bytes: [u8; 0] = [];
        assert_eq!(TestAccount::validate_discriminator(&bytes), Err(ProgramError::InvalidAccountData));
    }
}
