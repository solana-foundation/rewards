use pinocchio::{account::AccountView, Address, ProgramResult};
use pinocchio_token_2022::instructions::TransferChecked;

use crate::{
    events::ClaimedEvent,
    state::{DirectDistribution, DirectRecipient},
    traits::{
        AccountSerialize, ClaimTracker, Distribution, DistributionSigner, EventSerialize, InstructionData,
        VestingParams,
    },
    utils::{emit_event, get_current_timestamp, get_mint_decimals, resolve_claim_amount},
    ID,
};

use super::ClaimDirect;

pub fn process_claim_direct(
    _program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = ClaimDirect::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

    let current_ts = get_current_timestamp()?;

    let distribution_data = ix.accounts.distribution.try_borrow()?;
    let mut distribution = DirectDistribution::from_account(&distribution_data, ix.accounts.distribution, &ID)?;
    drop(distribution_data);
    Distribution::validate_mint(&distribution, ix.accounts.mint.address())?;

    let recipient_data = ix.accounts.recipient_account.try_borrow()?;
    let mut recipient = DirectRecipient::from_account(&recipient_data, ix.accounts.recipient_account, &ID)?;
    drop(recipient_data);

    recipient.validate_distribution(ix.accounts.distribution.address())?;
    recipient.validate_recipient(ix.accounts.recipient.address())?;

    let unlocked_amount = VestingParams::calculate_unlocked(&recipient, current_ts)?;
    let claimable_amount = ClaimTracker::claimable_amount(&recipient, unlocked_amount)?;
    let claim_amount = resolve_claim_amount(ix.data.amount, claimable_amount)?;

    ClaimTracker::add_claimed(&mut recipient, claim_amount)?;
    Distribution::add_claimed(&mut distribution, claim_amount)?;

    let mut recipient_data = ix.accounts.recipient_account.try_borrow_mut()?;
    recipient.write_to_slice(&mut recipient_data)?;
    drop(recipient_data);

    let mut distribution_data = ix.accounts.distribution.try_borrow_mut()?;
    distribution.write_to_slice(&mut distribution_data)?;
    drop(distribution_data);

    let decimals = get_mint_decimals(ix.accounts.mint)?;

    distribution.with_signer(|signers| {
        TransferChecked {
            from: ix.accounts.distribution_vault,
            mint: ix.accounts.mint,
            to: ix.accounts.recipient_token_account,
            authority: ix.accounts.distribution,
            amount: claim_amount,
            decimals,
            token_program: ix.accounts.token_program.address(),
        }
        .invoke_signed(signers)
    })?;

    let event = ClaimedEvent::new(*ix.accounts.distribution.address(), *ix.accounts.recipient.address(), claim_amount);
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
