use pinocchio::{account::AccountView, error::ProgramError, Address, ProgramResult};
use pinocchio_associated_token_account::instructions::CreateIdempotent;

use crate::{
    events::DistributionCreatedEvent,
    state::RewardPool,
    traits::{AccountSerialize, AccountSize, EventSerialize, InstructionData, PdaSeeds},
    utils::{create_pda_account, emit_event},
    ID,
};

use super::CreateContinuousPool;

pub fn process_create_continuous_pool(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = CreateContinuousPool::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

    let pool = RewardPool::new(
        ix.data.bump,
        ix.data.balance_source,
        ix.data.revocable,
        ix.data.clawback_ts,
        *ix.accounts.authority.address(),
        *ix.accounts.tracked_mint.address(),
        *ix.accounts.reward_mint.address(),
        *ix.accounts.seed.address(),
    );

    pool.validate_pda(ix.accounts.reward_pool, &ID, ix.data.bump)?;

    let bump_seed = [ix.data.bump];
    let pool_seeds = pool.seeds_with_bump(&bump_seed);
    let pool_seeds_array: [_; 6] = pool_seeds.try_into().map_err(|_| ProgramError::InvalidArgument)?;

    create_pda_account(ix.accounts.payer, RewardPool::LEN, &ID, ix.accounts.reward_pool, pool_seeds_array)?;

    let mut pool_data = ix.accounts.reward_pool.try_borrow_mut()?;
    pool.write_to_slice(&mut pool_data)?;
    drop(pool_data);

    CreateIdempotent {
        funding_account: ix.accounts.payer,
        account: ix.accounts.reward_vault,
        wallet: ix.accounts.reward_pool,
        mint: ix.accounts.reward_mint,
        system_program: ix.accounts.system_program,
        token_program: ix.accounts.reward_token_program,
    }
    .invoke()?;

    let event = DistributionCreatedEvent::continuous(
        *ix.accounts.authority.address(),
        *ix.accounts.reward_mint.address(),
        *ix.accounts.seed.address(),
        *ix.accounts.tracked_mint.address(),
        ix.data.balance_source.to_byte(),
        ix.data.clawback_ts,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
