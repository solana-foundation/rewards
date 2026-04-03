use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    events::MerkleRootSetEvent,
    state::RewardPool,
    traits::{AccountSerialize, EventSerialize, InstructionData},
    utils::emit_event,
    ID,
};

use super::SetContinuousMerkleRoot;

pub fn process_set_continuous_merkle_root(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = SetContinuousMerkleRoot::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

    let pool_data = ix.accounts.reward_pool.try_borrow()?;
    let mut pool = RewardPool::from_account(&pool_data, ix.accounts.reward_pool, &ID)?;
    drop(pool_data);

    pool.validate_authority(ix.accounts.authority.address())?;
    pool.validate_merkle_root_version(ix.data.root_version)?;

    pool.merkle_root = ix.data.merkle_root;
    pool.merkle_root_version = ix.data.root_version;

    let mut pool_data = ix.accounts.reward_pool.try_borrow_mut()?;
    pool.write_to_slice(&mut pool_data)?;
    drop(pool_data);

    let event = MerkleRootSetEvent::new(
        *ix.accounts.reward_pool.address(),
        *ix.accounts.authority.address(),
        ix.data.merkle_root,
        ix.data.root_version,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
