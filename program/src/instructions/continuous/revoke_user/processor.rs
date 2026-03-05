use pinocchio::{account::AccountView, error::ProgramError, Address, ProgramResult};
use pinocchio_token_2022::instructions::TransferChecked;

use crate::{
    errors::RewardsProgramError,
    events::RecipientRevokedEvent,
    state::{Revocation, RevocationSeeds, RewardPool, UserRewardAccount},
    traits::{AccountSerialize, AccountSize, EventSerialize, PdaSeeds},
    utils::{
        close_pda_account, create_pda_account, emit_event, get_mint_decimals, get_token_account_balance,
        is_pda_uninitialized, sync_user_balance, update_user_rewards, BalanceSource, RevokeMode,
    },
    ID,
};

use super::RevokeContinuousUser;

pub fn process_revoke_continuous_user(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = RevokeContinuousUser::try_from((instruction_data, accounts))?;

    let pool_data = ix.accounts.reward_pool.try_borrow()?;
    let mut pool = RewardPool::from_account(&pool_data, ix.accounts.reward_pool, &ID)?;
    drop(pool_data);

    pool.validate_authority(ix.accounts.authority.address())?;
    pool.validate_tracked_mint(ix.accounts.tracked_mint.address())?;
    pool.validate_reward_mint(ix.accounts.reward_mint.address())?;

    if ix.data.revoke_mode.is_disabled_by(pool.revocable) {
        return Err(RewardsProgramError::DistributionNotRevocable.into());
    }

    let user_data = ix.accounts.user_reward_account.try_borrow()?;
    let mut user = UserRewardAccount::from_account(
        &user_data,
        ix.accounts.user_reward_account,
        &ID,
        ix.accounts.reward_pool.address(),
        ix.accounts.user.address(),
    )?;
    drop(user_data);

    let revocation_seeds =
        RevocationSeeds { parent: *ix.accounts.reward_pool.address(), user: *ix.accounts.user.address() };
    let revocation_bump = revocation_seeds.validate_pda_address(ix.accounts.revocation_marker, &ID)?;

    if !is_pda_uninitialized(ix.accounts.revocation_marker) {
        return Err(RewardsProgramError::UserAlreadyRevoked.into());
    }

    update_user_rewards(&pool, &mut user)?;

    if pool.balance_source == BalanceSource::OnChain {
        let current_balance = get_token_account_balance(ix.accounts.user_tracked_token_account)?;
        sync_user_balance(&mut pool, &mut user, current_balance)?;
    }

    let rewards_transferred;
    let rewards_forfeited;

    match ix.data.revoke_mode {
        RevokeMode::NonVested => {
            rewards_transferred = user.accrued_rewards;
            rewards_forfeited = 0;

            if rewards_transferred > 0 {
                let decimals = get_mint_decimals(ix.accounts.reward_mint)?;

                pool.total_claimed =
                    pool.total_claimed.checked_add(rewards_transferred).ok_or(RewardsProgramError::MathOverflow)?;

                pool.with_signer(|signers| {
                    TransferChecked {
                        from: ix.accounts.reward_vault,
                        mint: ix.accounts.reward_mint,
                        to: ix.accounts.user_reward_token_account,
                        authority: ix.accounts.reward_pool,
                        amount: rewards_transferred,
                        decimals,
                        token_program: ix.accounts.reward_token_program.address(),
                    }
                    .invoke_signed(signers)
                })?;
            }
        }
        RevokeMode::Full => {
            rewards_transferred = 0;
            rewards_forfeited = user.accrued_rewards;

            if rewards_forfeited > 0 {
                let decimals = get_mint_decimals(ix.accounts.reward_mint)?;

                pool.total_claimed =
                    pool.total_claimed.checked_add(rewards_forfeited).ok_or(RewardsProgramError::MathOverflow)?;

                pool.with_signer(|signers| {
                    TransferChecked {
                        from: ix.accounts.reward_vault,
                        mint: ix.accounts.reward_mint,
                        to: ix.accounts.authority_reward_token_account,
                        authority: ix.accounts.reward_pool,
                        amount: rewards_forfeited,
                        decimals,
                        token_program: ix.accounts.reward_token_program.address(),
                    }
                    .invoke_signed(signers)
                })?;
            }
        }
    }

    pool.opted_in_supply =
        pool.opted_in_supply.checked_sub(user.last_known_balance).ok_or(RewardsProgramError::MathOverflow)?;

    let mut pool_data = ix.accounts.reward_pool.try_borrow_mut()?;
    pool.write_to_slice(&mut pool_data)?;
    drop(pool_data);

    close_pda_account(ix.accounts.user_reward_account, ix.accounts.rent_destination)?;

    let revocation_bump_seed = [revocation_bump];
    let revocation_pda_seeds = revocation_seeds.seeds_with_bump(&revocation_bump_seed);
    let revocation_pda_seeds_array: [_; 4] =
        revocation_pda_seeds.try_into().map_err(|_| ProgramError::InvalidArgument)?;

    create_pda_account(
        ix.accounts.payer,
        Revocation::LEN,
        &ID,
        ix.accounts.revocation_marker,
        revocation_pda_seeds_array,
    )?;

    let revocation = Revocation::new(revocation_bump);
    let mut revocation_data = ix.accounts.revocation_marker.try_borrow_mut()?;
    revocation.write_to_slice(&mut revocation_data)?;
    drop(revocation_data);

    let event = RecipientRevokedEvent::new(
        *ix.accounts.reward_pool.address(),
        *ix.accounts.user.address(),
        ix.data.revoke_mode,
        rewards_transferred,
        rewards_forfeited,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
