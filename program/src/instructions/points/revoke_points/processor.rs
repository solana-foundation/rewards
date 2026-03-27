use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    events::PointsRevokedEvent,
    state::{PointsConfig, UserPointsAccount},
    traits::{AccountSerialize, EventSerialize},
    utils::{close_pda_account, emit_event},
    ID,
};

use super::RevokePoints;

pub fn process_revoke_points(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = RevokePoints::try_from((instruction_data, accounts))?;

    // Parse and validate config
    let config_data = ix.accounts.points_config.try_borrow()?;
    let mut config = PointsConfig::from_account(&config_data, ix.accounts.points_config, &ID)?;
    drop(config_data);

    config.validate_authority(ix.accounts.authority.address())?;
    config.validate_revocable()?;

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

    let revoked_balance = user_account.balance;

    // Count revoked points as used
    if revoked_balance > 0 {
        config.add_used(revoked_balance)?;

        let mut config_data = ix.accounts.points_config.try_borrow_mut()?;
        config.write_to_slice(&mut config_data)?;
        drop(config_data);
    }

    // Close user account, return rent to destination
    close_pda_account(ix.accounts.user_points_account, ix.accounts.destination)?;

    let event = PointsRevokedEvent::new(
        *ix.accounts.points_config.address(),
        config.authority,
        config.seed,
        config.max_supply,
        config.transferable,
        config.revocable,
        config.total_issued,
        config.total_used,
        *ix.accounts.user.address(),
        revoked_balance,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
