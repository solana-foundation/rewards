use pinocchio::{account::AccountView, error::ProgramError, Address, ProgramResult};

use crate::{
    errors::RewardsProgramError,
    events::PointsTransferredEvent,
    state::{PointsConfig, UserPointsAccount, UserPointsAccountSeeds},
    traits::{AccountParse, AccountSerialize, AccountSize, EventSerialize, InstructionData, PdaSeeds},
    utils::{create_pda_account, emit_event, is_pda_uninitialized},
    ID,
};

use super::TransferPoints;

pub fn process_transfer_points(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = TransferPoints::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

    // Parse and validate config
    let config_data = ix.accounts.points_config.try_borrow()?;
    let config = PointsConfig::from_account(&config_data, ix.accounts.points_config, &ID)?;
    drop(config_data);

    config.validate_authority(ix.accounts.authority.address())?;
    config.validate_transferable()?;

    // Prevent self-transfers — writing from_account then to_account to the same
    // underlying account would silently clobber the first write.
    if ix.accounts.from_user.address() == ix.accounts.to_user.address() {
        return Err(RewardsProgramError::PointsSelfTransferNotAllowed.into());
    }

    // Validate from_user points PDA
    let from_seeds = UserPointsAccountSeeds {
        points_config: *ix.accounts.points_config.address(),
        user: *ix.accounts.from_user.address(),
    };
    from_seeds.validate_pda_address(ix.accounts.from_user_points, &ID)?;

    // Parse from user account
    let from_data = ix.accounts.from_user_points.try_borrow()?;
    let mut from_account = UserPointsAccount::parse_from_bytes(&from_data)?;
    drop(from_data);

    from_account.validate_balance(ix.data.quantity)?;

    // Validate to_user points PDA
    let to_seeds = UserPointsAccountSeeds {
        points_config: *ix.accounts.points_config.address(),
        user: *ix.accounts.to_user.address(),
    };
    to_seeds.validate_pda(ix.accounts.to_user_points, &ID, ix.data.to_user_points_bump)?;

    // Create to_user account if needed
    let mut to_account = if is_pda_uninitialized(ix.accounts.to_user_points) {
        let bump_seed = [ix.data.to_user_points_bump];
        let pda_seeds = to_seeds.seeds_with_bump(&bump_seed);
        let pda_seeds_array: [_; 4] = pda_seeds.try_into().map_err(|_| ProgramError::InvalidArgument)?;

        create_pda_account(
            ix.accounts.payer,
            UserPointsAccount::LEN,
            &ID,
            ix.accounts.to_user_points,
            pda_seeds_array,
        )?;

        UserPointsAccount::new(ix.data.to_user_points_bump)
    } else {
        let to_data = ix.accounts.to_user_points.try_borrow()?;
        let account = UserPointsAccount::parse_from_bytes(&to_data)?;
        drop(to_data);
        account
    };

    // Transfer balances
    from_account.balance =
        from_account.balance.checked_sub(ix.data.quantity).ok_or(RewardsProgramError::MathOverflow)?;

    to_account.balance = to_account.balance.checked_add(ix.data.quantity).ok_or(RewardsProgramError::MathOverflow)?;

    // Write updated state
    let mut from_account_data = ix.accounts.from_user_points.try_borrow_mut()?;
    from_account.write_to_slice(&mut from_account_data)?;
    drop(from_account_data);

    let mut to_account_data = ix.accounts.to_user_points.try_borrow_mut()?;
    to_account.write_to_slice(&mut to_account_data)?;
    drop(to_account_data);

    let event = PointsTransferredEvent::new(
        *ix.accounts.points_config.address(),
        config.authority,
        config.seed,
        config.max_supply,
        config.transferable,
        config.revocable,
        config.total_issued,
        config.total_used,
        *ix.accounts.from_user.address(),
        *ix.accounts.to_user.address(),
        ix.data.quantity,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
