use pinocchio::{account::AccountView, error::ProgramError, Address, ProgramResult};
use pinocchio_token_2022::instructions::TransferChecked;

use crate::{
    errors::RewardsProgramError,
    events::RecipientAddedEvent,
    state::{DirectDistribution, DirectRecipient},
    traits::{AccountSerialize, Distribution, EventSerialize, InstructionData, PdaSeeds},
    utils::{create_pda_account, emit_event, get_mint_decimals},
    ID,
};

use super::AddDirectRecipient;

pub fn process_add_direct_recipient(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = AddDirectRecipient::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

    let distribution_data = ix.accounts.distribution.try_borrow()?;
    let mut distribution = DirectDistribution::from_account(&distribution_data, ix.accounts.distribution, &ID)?;
    drop(distribution_data);

    Distribution::validate_authority(&distribution, ix.accounts.authority.address())?;
    Distribution::validate_mint(&distribution, ix.accounts.mint.address())?;

    let new_total_allocated =
        distribution.total_allocated.checked_add(ix.data.amount).ok_or(RewardsProgramError::MathOverflow)?;

    let recipient = DirectRecipient::new(
        ix.data.bump,
        *ix.accounts.distribution.address(),
        *ix.accounts.recipient.address(),
        *ix.accounts.payer.address(),
        ix.data.amount,
        ix.data.schedule,
    );

    recipient.validate_pda(ix.accounts.recipient_account, &ID, ix.data.bump)?;

    let bump_seed = [ix.data.bump];
    let recipient_seeds = recipient.seeds_with_bump(&bump_seed);
    let recipient_seeds_array: [_; 4] = recipient_seeds.try_into().map_err(|_| ProgramError::InvalidArgument)?;

    create_pda_account(
        ix.accounts.payer,
        DirectRecipient::calculate_account_size(&ix.data.schedule),
        &ID,
        ix.accounts.recipient_account,
        recipient_seeds_array,
    )?;

    let mut recipient_data = ix.accounts.recipient_account.try_borrow_mut()?;
    recipient.write_to_slice(&mut recipient_data)?;
    drop(recipient_data);

    distribution.total_allocated = new_total_allocated;
    let mut distribution_data = ix.accounts.distribution.try_borrow_mut()?;
    distribution.write_to_slice(&mut distribution_data)?;
    drop(distribution_data);

    let decimals = get_mint_decimals(ix.accounts.mint)?;

    TransferChecked {
        from: ix.accounts.authority_token_account,
        mint: ix.accounts.mint,
        to: ix.accounts.distribution_vault,
        authority: ix.accounts.authority,
        amount: ix.data.amount,
        decimals,
        token_program: ix.accounts.token_program.address(),
    }
    .invoke()?;

    let event = RecipientAddedEvent::new(
        *ix.accounts.distribution.address(),
        *ix.accounts.recipient.address(),
        ix.data.amount,
        ix.data.schedule,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
