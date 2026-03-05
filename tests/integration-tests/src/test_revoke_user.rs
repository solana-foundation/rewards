use rewards_program_client::types::RevokeMode;
use solana_sdk::signature::Signer;

use crate::fixtures::{RevokeContinuousUserFixture, RevokeContinuousUserSetup};
use crate::utils::{
    assert_account_closed, assert_rewards_error, find_event_authority_pda, get_reward_pool, test_empty_data,
    test_missing_signer, test_not_writable, test_truncated_data, test_wrong_current_program, test_wrong_system_program,
    RewardsError, TestContext, TestInstruction,
};

// ─── Generic validation tests ───

#[test]
fn test_revoke_user_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<RevokeContinuousUserFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_revoke_user_pool_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<RevokeContinuousUserFixture>(&mut ctx, 2);
}

#[test]
fn test_revoke_user_user_reward_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<RevokeContinuousUserFixture>(&mut ctx, 3);
}

#[test]
fn test_revoke_user_revocation_marker_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<RevokeContinuousUserFixture>(&mut ctx, 4);
}

#[test]
fn test_revoke_user_vault_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<RevokeContinuousUserFixture>(&mut ctx, 8);
}

#[test]
fn test_revoke_user_user_reward_token_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<RevokeContinuousUserFixture>(&mut ctx, 9);
}

#[test]
fn test_revoke_user_wrong_system_program() {
    let mut ctx = TestContext::new();
    test_wrong_system_program::<RevokeContinuousUserFixture>(&mut ctx);
}

#[test]
fn test_revoke_user_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<RevokeContinuousUserFixture>(&mut ctx);
}

#[test]
fn test_revoke_user_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<RevokeContinuousUserFixture>(&mut ctx);
}

#[test]
fn test_revoke_user_truncated_data() {
    let mut ctx = TestContext::new();
    test_truncated_data::<RevokeContinuousUserFixture>(&mut ctx);
}

// ─── Custom error tests ───

#[test]
fn test_revoke_user_unauthorized_authority() {
    let mut ctx = TestContext::new();
    let setup = RevokeContinuousUserSetup::new(&mut ctx);

    let wrong_authority = ctx.create_funded_keypair();
    let pool_setup = &setup.distribute_setup.opt_in_setup.pool_setup;
    let user = &setup.distribute_setup.opt_in_setup.user;
    let user_reward_pda = &setup.distribute_setup.opt_in_setup.user_reward_pda;
    let user_tracked_ta = &setup.distribute_setup.opt_in_setup.user_tracked_token_account;

    let (event_authority, _) = find_event_authority_pda();
    let (revocation_pda, _) = crate::utils::find_revocation_pda(&pool_setup.reward_pool_pda, &user.pubkey());

    let wrong_authority_reward_ta =
        ctx.create_token_account(&wrong_authority.pubkey(), &pool_setup.reward_mint.pubkey());

    let mut builder = rewards_program_client::instructions::RevokeContinuousUserBuilder::new();
    builder
        .authority(wrong_authority.pubkey())
        .payer(wrong_authority.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .user_reward_account(*user_reward_pda)
        .revocation_marker(revocation_pda)
        .user(user.pubkey())
        .rent_destination(user.pubkey())
        .user_tracked_token_account(*user_tracked_ta)
        .reward_vault(pool_setup.reward_vault)
        .user_reward_token_account(setup.user_reward_token_account)
        .authority_reward_token_account(wrong_authority_reward_ta)
        .tracked_mint(pool_setup.tracked_mint.pubkey())
        .reward_mint(pool_setup.reward_mint.pubkey())
        .tracked_token_program(spl_token_interface::ID)
        .reward_token_program(pool_setup.reward_token_program)
        .event_authority(event_authority)
        .revoke_mode(RevokeMode::NonVested);

    let ix = TestInstruction {
        instruction: builder.instruction(),
        signers: vec![wrong_authority],
        name: "RevokeContinuousUser",
    };
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}

#[test]
fn test_revoke_user_not_revocable() {
    let mut ctx = TestContext::new();
    let setup = RevokeContinuousUserSetup::new_with_revocable(&mut ctx, 0);
    let ix = setup.build_instruction(&ctx, RevokeMode::NonVested);

    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::DistributionNotRevocable);
}

#[test]
fn test_revoke_user_non_vested_rejected_when_only_full_bit_set() {
    let mut ctx = TestContext::new();
    let setup = RevokeContinuousUserSetup::new_with_revocable(&mut ctx, 2);
    let ix = setup.build_instruction(&ctx, RevokeMode::NonVested);

    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::DistributionNotRevocable);
}

#[test]
fn test_revoke_user_full_rejected_when_only_non_vested_bit_set() {
    let mut ctx = TestContext::new();
    let setup = RevokeContinuousUserSetup::new_with_revocable(&mut ctx, 1);
    let ix = setup.build_instruction(&ctx, RevokeMode::Full);

    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::DistributionNotRevocable);
}

#[test]
fn test_revoke_user_already_revoked() {
    let mut ctx = TestContext::new();
    let setup = RevokeContinuousUserSetup::new(&mut ctx);
    let ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    ix.send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &setup.distribute_setup.opt_in_setup.user_reward_pda);

    let pool_setup = &setup.distribute_setup.opt_in_setup.pool_setup;
    let pool = get_reward_pool(&ctx, &pool_setup.reward_pool_pda);
    assert_eq!(pool.opted_in_supply, 0);
}

#[test]
fn test_revoke_user_success_non_vested() {
    let mut ctx = TestContext::new();
    let setup = RevokeContinuousUserSetup::new(&mut ctx);
    let ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    ix.send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &setup.distribute_setup.opt_in_setup.user_reward_pda);

    let pool_setup = &setup.distribute_setup.opt_in_setup.pool_setup;
    let pool = get_reward_pool(&ctx, &pool_setup.reward_pool_pda);
    assert_eq!(pool.opted_in_supply, 0);
}

#[test]
fn test_revoke_user_success_full() {
    let mut ctx = TestContext::new();
    let setup = RevokeContinuousUserSetup::new(&mut ctx);
    let ix = setup.build_instruction(&ctx, RevokeMode::Full);
    ix.send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &setup.distribute_setup.opt_in_setup.user_reward_pda);
}
