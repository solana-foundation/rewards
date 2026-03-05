use pinocchio::{account::AccountView, Address, ProgramResult};
use pinocchio_token_2022::instructions::TransferChecked;

use crate::{
    errors::RewardsProgramError,
    events::RecipientRevokedEvent,
    state::{DirectDistribution, DirectRecipient},
    traits::{AccountSerialize, Distribution, DistributionSigner, EventSerialize, InstructionData, VestingParams},
    utils::{close_pda_account, emit_event, get_current_timestamp, get_mint_decimals, RevokeMode},
    ID,
};

use super::RevokeDirectRecipient;

pub fn process_revoke_direct_recipient(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = RevokeDirectRecipient::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

    let distribution_data = ix.accounts.distribution.try_borrow()?;
    let mut distribution = DirectDistribution::from_account(&distribution_data, ix.accounts.distribution, &ID)?;
    drop(distribution_data);

    distribution.validate_authority(ix.accounts.authority.address())?;

    if ix.data.revoke_mode.is_disabled_by(distribution.revocable) {
        return Err(RewardsProgramError::DistributionNotRevocable.into());
    }

    let recipient_data = ix.accounts.recipient_account.try_borrow()?;
    let recipient = DirectRecipient::from_account(&recipient_data, ix.accounts.recipient_account, &ID)?;
    drop(recipient_data);

    recipient.validate_distribution(ix.accounts.distribution.address())?;
    recipient.validate_recipient(ix.accounts.recipient.address())?;

    if &recipient.payer != ix.accounts.original_payer.address() {
        return Err(pinocchio::error::ProgramError::InvalidAccountData);
    }

    let current_ts = get_current_timestamp()?;
    let vested_amount = VestingParams::calculate_unlocked(&recipient, current_ts)?;
    let vested_unclaimed =
        vested_amount.checked_sub(recipient.claimed_amount).ok_or(RewardsProgramError::MathOverflow)?;
    let unvested = recipient.total_amount.checked_sub(vested_amount).ok_or(RewardsProgramError::MathOverflow)?;

    let decimals = get_mint_decimals(ix.accounts.mint)?;

    let (vested_transferred, total_freed) = match ix.data.revoke_mode {
        RevokeMode::NonVested => {
            if vested_unclaimed > 0 {
                distribution.with_signer(|signers| {
                    TransferChecked {
                        from: ix.accounts.distribution_vault,
                        mint: ix.accounts.mint,
                        to: ix.accounts.recipient_token_account,
                        authority: ix.accounts.distribution,
                        amount: vested_unclaimed,
                        decimals,
                        token_program: ix.accounts.token_program.address(),
                    }
                    .invoke_signed(signers)
                })?;
            }

            distribution.total_allocated =
                distribution.total_allocated.checked_sub(unvested).ok_or(RewardsProgramError::MathOverflow)?;
            distribution.total_claimed =
                distribution.total_claimed.checked_add(vested_unclaimed).ok_or(RewardsProgramError::MathOverflow)?;

            (vested_unclaimed, unvested)
        }
        RevokeMode::Full => {
            let total_freed = unvested.checked_add(vested_unclaimed).ok_or(RewardsProgramError::MathOverflow)?;

            distribution.total_allocated =
                distribution.total_allocated.checked_sub(total_freed).ok_or(RewardsProgramError::MathOverflow)?;

            (0, total_freed)
        }
    };

    if total_freed > 0 {
        distribution.with_signer(|signers| {
            TransferChecked {
                from: ix.accounts.distribution_vault,
                mint: ix.accounts.mint,
                to: ix.accounts.authority_token_account,
                authority: ix.accounts.distribution,
                amount: total_freed,
                decimals,
                token_program: ix.accounts.token_program.address(),
            }
            .invoke_signed(signers)
        })?;
    }

    let mut distribution_data = ix.accounts.distribution.try_borrow_mut()?;
    distribution.write_to_slice(&mut distribution_data)?;
    drop(distribution_data);

    close_pda_account(ix.accounts.recipient_account, ix.accounts.original_payer)?;

    let event = RecipientRevokedEvent::new(
        *ix.accounts.distribution.address(),
        *ix.accounts.recipient.address(),
        ix.data.revoke_mode,
        vested_transferred,
        total_freed,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
