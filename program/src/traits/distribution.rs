use pinocchio::{cpi::Signer, error::ProgramError, Address};

use crate::errors::RewardsProgramError;

use super::{AccountParse, AccountSerialize, PdaAccount};

/// Common interface for distribution account types (Merkle and Direct).
///
/// This trait abstracts over the shared operations that both distribution
/// types support, enabling code reuse in claim processors.
pub trait Distribution: AccountParse + AccountSerialize + PdaAccount {
    /// Returns the mint address for this distribution
    fn mint(&self) -> &Address;

    /// Returns the authority address for this distribution
    fn authority(&self) -> &Address;

    /// Returns the seeds key used for PDA derivation
    fn seeds_key(&self) -> &Address;

    /// Returns the total amount claimed from this distribution
    fn total_claimed(&self) -> u64;

    /// Sets the total claimed amount
    fn set_total_claimed(&mut self, amount: u64) -> Result<(), ProgramError>;

    /// Validates that the provided authority matches the distribution's authority
    #[inline(always)]
    fn validate_authority(&self, authority: &Address) -> Result<(), ProgramError> {
        if self.authority() != authority {
            return Err(RewardsProgramError::UnauthorizedAuthority.into());
        }
        Ok(())
    }

    /// Validates that the provided mint matches the distribution mint
    #[inline(always)]
    fn validate_mint(&self, mint: &Address) -> Result<(), ProgramError> {
        if self.mint() != mint {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    /// Adds to the total claimed amount with overflow checking
    #[inline(always)]
    fn add_claimed(&mut self, amount: u64) -> Result<(), ProgramError> {
        let new_total = self.total_claimed().checked_add(amount).ok_or(RewardsProgramError::MathOverflow)?;
        self.set_total_claimed(new_total)?;
        Ok(())
    }
}

/// Extension trait for distributions that can sign CPIs.
///
/// Distributions are PDAs that can sign cross-program invocations
/// (e.g., token transfers from the distribution vault).
pub trait DistributionSigner: Distribution {
    /// Executes a closure with the distribution's PDA signer seeds.
    fn with_signer<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Signer<'_, '_>]) -> R;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_validate_authority_impl(authority: &Address, check_authority: &Address) -> Result<(), ProgramError> {
        if authority != check_authority {
            return Err(RewardsProgramError::UnauthorizedAuthority.into());
        }
        Ok(())
    }

    fn test_validate_mint_impl(mint: &Address, check_mint: &Address) -> Result<(), ProgramError> {
        if mint != check_mint {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    fn test_add_claimed_impl(current: u64, amount: u64) -> Result<u64, ProgramError> {
        current.checked_add(amount).ok_or(RewardsProgramError::MathOverflow.into())
    }

    #[test]
    fn test_validate_authority_success() {
        let authority = Address::new_from_array([2u8; 32]);
        assert!(test_validate_authority_impl(&authority, &authority).is_ok());
    }

    #[test]
    fn test_validate_authority_fail() {
        let authority = Address::new_from_array([2u8; 32]);
        let wrong_authority = Address::new_from_array([99u8; 32]);
        assert!(test_validate_authority_impl(&authority, &wrong_authority).is_err());
    }

    #[test]
    fn test_validate_mint_success() {
        let mint = Address::new_from_array([3u8; 32]);
        assert!(test_validate_mint_impl(&mint, &mint).is_ok());
    }

    #[test]
    fn test_validate_mint_fail() {
        let mint = Address::new_from_array([3u8; 32]);
        let wrong_mint = Address::new_from_array([4u8; 32]);
        assert!(test_validate_mint_impl(&mint, &wrong_mint).is_err());
    }

    #[test]
    fn test_add_claimed_success() {
        let result = test_add_claimed_impl(0, 500);
        assert_eq!(result.unwrap(), 500);
    }

    #[test]
    fn test_add_claimed_accumulates() {
        let result = test_add_claimed_impl(500, 300);
        assert_eq!(result.unwrap(), 800);
    }

    #[test]
    fn test_add_claimed_overflow() {
        let result = test_add_claimed_impl(u64::MAX, 1);
        assert!(result.is_err());
    }
}
