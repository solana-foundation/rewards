use pinocchio::{account::AccountView, error::ProgramError, Address, ProgramResult};
use pinocchio_token_2022::instructions::{CloseAccount, TransferChecked};

use crate::{
    errors::RewardsProgramError,
    events::DistributionClosedEvent,
    state::{DirectDistribution, DirectDistributionTombstone, DirectDistributionTombstoneSeeds},
    traits::{AccountSerialize, AccountSize, Distribution, DistributionSigner, EventSerialize, PdaSeeds},
    utils::{
        close_pda_account, create_pda_account, emit_event, get_current_timestamp, get_mint_decimals,
        get_token_account_balance, is_pda_uninitialized,
    },
    ID,
};

use super::CloseDirectDistribution;

pub fn process_close_direct_distribution(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = CloseDirectDistribution::try_from((instruction_data, accounts))?;

    let distribution_data = ix.accounts.distribution.try_borrow()?;
    let distribution = DirectDistribution::from_account(&distribution_data, ix.accounts.distribution, &ID)?;
    distribution.validate_authority(ix.accounts.authority.address())?;
    distribution.validate_mint(ix.accounts.mint.address())?;
    let tombstone_seeds = DirectDistributionTombstoneSeeds { distribution: *ix.accounts.distribution.address() };
    let tombstone_bump = tombstone_seeds.validate_pda_address(ix.accounts.tombstone, &ID)?;
    if !is_pda_uninitialized(ix.accounts.tombstone) {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if distribution.clawback_ts != 0 {
        let current_ts = get_current_timestamp()?;
        if current_ts < distribution.clawback_ts {
            return Err(RewardsProgramError::ClawbackNotReached.into());
        }
    }

    let remaining_amount = get_token_account_balance(ix.accounts.distribution_vault)?;
    let decimals = get_mint_decimals(ix.accounts.mint)?;

    if remaining_amount > 0 {
        distribution.with_signer(|signers| {
            TransferChecked {
                from: ix.accounts.distribution_vault,
                mint: ix.accounts.mint,
                to: ix.accounts.authority_token_account,
                authority: ix.accounts.distribution,
                amount: remaining_amount,
                decimals,
                token_program: ix.accounts.token_program.address(),
            }
            .invoke_signed(signers)
        })?;
    }

    distribution.with_signer(|signers| {
        CloseAccount {
            account: ix.accounts.distribution_vault,
            destination: ix.accounts.authority,
            authority: ix.accounts.distribution,
            token_program: ix.accounts.token_program.address(),
        }
        .invoke_signed(signers)
    })?;

    drop(distribution_data);

    let tombstone_bump_seed = [tombstone_bump];
    let tombstone_pda_seeds = tombstone_seeds.seeds_with_bump(&tombstone_bump_seed);
    let tombstone_pda_seeds_array: [_; 3] =
        tombstone_pda_seeds.try_into().map_err(|_| ProgramError::InvalidArgument)?;
    create_pda_account(
        ix.accounts.authority,
        DirectDistributionTombstone::LEN,
        &ID,
        ix.accounts.tombstone,
        tombstone_pda_seeds_array,
    )?;

    let tombstone = DirectDistributionTombstone::new(tombstone_bump);
    let mut tombstone_data = ix.accounts.tombstone.try_borrow_mut()?;
    tombstone.write_to_slice(&mut tombstone_data)?;
    drop(tombstone_data);

    close_pda_account(ix.accounts.distribution, ix.accounts.authority)?;

    let event = DistributionClosedEvent::new(*ix.accounts.distribution.address(), remaining_amount);
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
