use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    errors::RewardsProgramError,
    events::PointsAccountClosedEvent,
    state::{PointsConfig, UserPointsAccount},
    traits::EventSerialize,
    utils::{close_pda_account, emit_event},
    ID,
};

use super::ClosePointsAccount;

pub fn process_close_points_account(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = ClosePointsAccount::try_from((instruction_data, accounts))?;

    // Parse and validate config
    let config_data = ix.accounts.points_config.try_borrow()?;
    let config = PointsConfig::from_account(&config_data, ix.accounts.points_config, &ID)?;
    drop(config_data);

    config.validate_authority(ix.accounts.authority.address())?;

    // Parse and validate user points account
    let user_data = ix.accounts.user_points_account.try_borrow()?;
    let user_account = UserPointsAccount::from_account(
        &user_data,
        ix.accounts.user_points_account,
        &ID,
        ix.accounts.points_config.address(),
        ix.accounts.user.address(),
    )?;
    drop(user_data);

    // Require zero balance to close
    if user_account.balance != 0 {
        return Err(RewardsProgramError::PointsBalanceNotZero.into());
    }

    close_pda_account(ix.accounts.user_points_account, ix.accounts.destination)?;

    let event = PointsAccountClosedEvent::new(*ix.accounts.points_config.address(), *ix.accounts.user.address());
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
