use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    events::ClaimClosedEvent,
    state::MerkleClaim,
    traits::{EventSerialize, InstructionData},
    utils::{close_pda_account, emit_event, verify_system_account},
    ID,
};

use super::CloseMerkleClaim;

pub fn process_close_merkle_claim(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = CloseMerkleClaim::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

    // Distribution must be closed (owner = system program means account was deleted)
    verify_system_account(ix.accounts.distribution)?;

    let claim_data = ix.accounts.claim_account.try_borrow()?;
    let _claim = MerkleClaim::from_account(
        &claim_data,
        ix.accounts.claim_account,
        &ID,
        ix.accounts.distribution.address(),
        ix.accounts.claimant.address(),
    )?;
    drop(claim_data);

    // Close the claim account and return rent to claimant
    close_pda_account(ix.accounts.claim_account, ix.accounts.claimant)?;

    let event = ClaimClosedEvent::new(*ix.accounts.distribution.address(), *ix.accounts.claimant.address());
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
