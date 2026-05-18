use pinocchio::{account::AccountView, error::ProgramError, Address, ProgramResult};

use crate::{
    errors::RewardsProgramError,
    events::ClaimClosedEvent,
    state::{DirectDistribution, DirectRecipient},
    traits::{EventSerialize, InstructionData},
    utils::{close_pda_account, emit_event},
    ID,
};

use super::CloseDirectRecipient;

pub fn process_close_direct_recipient(
    _program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = CloseDirectRecipient::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

    // The distribution PDA always lives on: active as `DirectDistribution`,
    // permanently closed as a compact `DirectDistributionClosed` marker.
    let distribution_closed = DirectDistribution::is_closed(ix.accounts.distribution, &ID)?;

    if !distribution_closed {
        let distribution_data = ix.accounts.distribution.try_borrow()?;
        let _distribution = DirectDistribution::from_account(&distribution_data, ix.accounts.distribution, &ID)?;
        drop(distribution_data);
    }

    let recipient_data = ix.accounts.recipient_account.try_borrow()?;
    let recipient = DirectRecipient::from_account(&recipient_data, ix.accounts.recipient_account, &ID)?;
    drop(recipient_data);

    recipient.validate_distribution(ix.accounts.distribution.address())?;
    recipient.validate_recipient(ix.accounts.recipient.address())?;

    // Validate that the payer account matches the one stored in the recipient
    if &recipient.payer != ix.accounts.original_payer.address() {
        return Err(ProgramError::InvalidAccountData);
    }

    if !distribution_closed && recipient.claimed_amount < recipient.total_amount {
        return Err(RewardsProgramError::ClaimNotFullyVested.into());
    }

    // Return rent to the original payer who created this recipient account
    close_pda_account(ix.accounts.recipient_account, ix.accounts.original_payer)?;

    let event = ClaimClosedEvent::new(*ix.accounts.distribution.address(), *ix.accounts.recipient.address());
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
