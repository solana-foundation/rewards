use pinocchio::{account::AccountView, Address, ProgramResult};
use pinocchio_token_2022::instructions::CloseAccount;

use crate::{
    errors::RewardsProgramError,
    events::PointsAccountClosedEvent,
    state::{PointsConfig, PointsMintSeeds},
    traits::{EventSerialize, PdaSeeds},
    utils::{emit_event, get_token_account_balance},
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

    // Validate points mint PDA
    let mint_seeds = PointsMintSeeds { points_config: *ix.accounts.points_config.address() };
    mint_seeds.validate_pda(ix.accounts.points_mint, &ID, config.mint_bump)?;

    // Require zero balance before closing
    let balance = get_token_account_balance(ix.accounts.user_token_account)?;
    if balance != 0 {
        return Err(RewardsProgramError::PointsBalanceNotZero.into());
    }

    // Close the token account — user signs as ATA owner, rent refunded to user
    CloseAccount {
        account: ix.accounts.user_token_account,
        destination: ix.accounts.user,
        authority: ix.accounts.user,
        token_program: ix.accounts.token_2022_program.address(),
    }
    .invoke()?;

    let event = PointsAccountClosedEvent::new(
        *ix.accounts.points_config.address(),
        config.authority,
        config.seed,
        config.transferable,
        config.revocable,
        *ix.accounts.user.address(),
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
