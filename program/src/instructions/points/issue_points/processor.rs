use pinocchio::{account::AccountView, error::ProgramError, Address, ProgramResult};

use crate::{
    errors::RewardsProgramError,
    events::PointsIssuedEvent,
    state::{PointsConfig, UserPointsAccount, UserPointsAccountSeeds},
    traits::{AccountParse, AccountSerialize, AccountSize, EventSerialize, InstructionData, PdaSeeds},
    utils::{create_pda_account, emit_event, is_pda_uninitialized},
    ID,
};

use super::IssuePoints;

pub fn process_issue_points(_program_id: &Address, accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
    let ix = IssuePoints::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

    // Parse and validate config
    let config_data = ix.accounts.points_config.try_borrow()?;
    let mut config = PointsConfig::from_account(&config_data, ix.accounts.points_config, &ID)?;
    drop(config_data);

    config.validate_authority(ix.accounts.authority.address())?;
    config.validate_max_supply(ix.data.quantity)?;

    // Validate user points PDA
    let user_seeds = UserPointsAccountSeeds {
        points_config: *ix.accounts.points_config.address(),
        user: *ix.accounts.user.address(),
    };
    user_seeds.validate_pda(ix.accounts.user_points_account, &ID, ix.data.user_points_bump)?;

    // Idempotent: create user account inline on first issue (matches ClaimMerkle pattern).
    let mut user_account = if is_pda_uninitialized(ix.accounts.user_points_account) {
        let bump_seed = [ix.data.user_points_bump];
        let pda_seeds = user_seeds.seeds_with_bump(&bump_seed);
        let pda_seeds_array: [_; 4] = pda_seeds.try_into().map_err(|_| ProgramError::InvalidArgument)?;

        create_pda_account(
            ix.accounts.payer,
            UserPointsAccount::LEN,
            &ID,
            ix.accounts.user_points_account,
            pda_seeds_array,
        )?;

        UserPointsAccount::new(ix.data.user_points_bump)
    } else {
        let user_data = ix.accounts.user_points_account.try_borrow()?;
        let account = UserPointsAccount::parse_from_bytes(&user_data)?;
        drop(user_data);
        account
    };

    // Update balances
    user_account.balance =
        user_account.balance.checked_add(ix.data.quantity).ok_or(RewardsProgramError::MathOverflow)?;

    config.total_issued = config.total_issued.checked_add(ix.data.quantity).ok_or(RewardsProgramError::MathOverflow)?;

    // Write updated state
    let mut user_account_data = ix.accounts.user_points_account.try_borrow_mut()?;
    user_account.write_to_slice(&mut user_account_data)?;
    drop(user_account_data);

    let mut config_account_data = ix.accounts.points_config.try_borrow_mut()?;
    config.write_to_slice(&mut config_account_data)?;
    drop(config_account_data);

    let event = PointsIssuedEvent::new(
        *ix.accounts.points_config.address(),
        config.authority,
        config.seed,
        config.max_supply,
        config.transferable,
        config.revocable,
        config.total_issued,
        config.total_used,
        *ix.accounts.user.address(),
        ix.data.quantity,
        user_account.balance,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
