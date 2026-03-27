use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    events::PointsConfigClosedEvent,
    state::PointsConfig,
    traits::EventSerialize,
    utils::{close_pda_account, emit_event},
    ID,
};

use super::ClosePointsConfig;

pub fn process_close_points_config(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = ClosePointsConfig::try_from((instruction_data, accounts))?;

    let config_data = ix.accounts.points_config.try_borrow()?;
    let config = PointsConfig::from_account(&config_data, ix.accounts.points_config, &ID)?;
    drop(config_data);

    config.validate_authority(ix.accounts.authority.address())?;

    // Intentionally does not check outstanding balances. Authority has full discretion
    // to close the config at any time. Use revoke_points to clean up user accounts first.
    close_pda_account(ix.accounts.points_config, ix.accounts.destination)?;

    let event = PointsConfigClosedEvent::new(
        *ix.accounts.points_config.address(),
        config.authority,
        config.seed,
        config.max_supply,
        config.transferable,
        config.revocable,
        config.total_issued,
        config.total_used,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
