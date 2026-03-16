use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        validate_associated_token_account, verify_current_program, verify_current_program_account,
        verify_event_authority, verify_owned_by, verify_readonly, verify_signer, verify_token_program, verify_writable,
        ZK_ELGAMAL_PROOF_PROGRAM_ID,
    },
};

pub struct CloseContinuousPoolAccounts<'a> {
    pub authority: &'a AccountView,
    pub reward_pool: &'a AccountView,
    pub reward_mint: &'a AccountView,
    pub reward_vault: &'a AccountView,
    pub authority_token_account: &'a AccountView,
    pub reward_token_program: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
    /// Required when `pool.confidential_rewards != 0` and unclaimed rewards remain.
    pub equality_proof_context: Option<&'a AccountView>,
    /// Required when `pool.confidential_rewards != 0` and unclaimed rewards remain.
    pub range_proof_context: Option<&'a AccountView>,
    /// Required when `pool.confidential_rewards != 0` and unclaimed rewards remain.
    pub zero_ciphertext_proof_context: Option<&'a AccountView>,
}

impl<'a> TryFrom<&'a [AccountView]> for CloseContinuousPoolAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 8 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let authority = &accounts[0];
        let reward_pool = &accounts[1];
        let reward_mint = &accounts[2];
        let reward_vault = &accounts[3];
        let authority_token_account = &accounts[4];
        let reward_token_program = &accounts[5];
        let event_authority = &accounts[6];
        let program = &accounts[7];

        verify_signer(authority, true)?;

        verify_writable(reward_pool, true)?;
        verify_writable(reward_vault, true)?;
        verify_writable(authority_token_account, true)?;

        verify_readonly(reward_mint)?;

        verify_current_program_account(reward_pool)?;

        verify_token_program(reward_token_program)?;
        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        verify_owned_by(reward_mint, reward_token_program.address())?;

        validate_associated_token_account(reward_vault, reward_pool.address(), reward_mint, reward_token_program)?;

        let (equality_proof_context, range_proof_context, zero_ciphertext_proof_context) = if accounts.len() >= 11 {
            let eq_ctx = &accounts[8];
            let range_ctx = &accounts[9];
            let zero_ctx = &accounts[10];
            verify_owned_by(eq_ctx, &ZK_ELGAMAL_PROOF_PROGRAM_ID)?;
            verify_owned_by(range_ctx, &ZK_ELGAMAL_PROOF_PROGRAM_ID)?;
            verify_owned_by(zero_ctx, &ZK_ELGAMAL_PROOF_PROGRAM_ID)?;
            (Some(eq_ctx), Some(range_ctx), Some(zero_ctx))
        } else {
            (None, None, None)
        };

        Ok(Self {
            authority,
            reward_pool,
            reward_mint,
            reward_vault,
            authority_token_account,
            reward_token_program,
            event_authority,
            program,
            equality_proof_context,
            range_proof_context,
            zero_ciphertext_proof_context,
        })
    }
}

impl<'a> InstructionAccounts<'a> for CloseContinuousPoolAccounts<'a> {}
