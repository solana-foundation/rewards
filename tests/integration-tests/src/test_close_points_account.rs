use solana_sdk::signer::Signer;

use crate::fixtures::{ClosePointsAccountFixture, ClosePointsAccountSetup, InitPointsSetup, IssuePointsSetup};
use crate::utils::{
    assert_account_closed, assert_rewards_error, find_event_authority_pda, find_user_points_pda, test_empty_data,
    test_missing_signer, test_not_writable, test_wrong_current_program, RewardsError, TestContext, TestInstruction,
};

// ── Generic validation tests ────────────────────────────────────────────────

#[test]
fn test_close_points_account_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<ClosePointsAccountFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_close_points_account_user_points_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClosePointsAccountFixture>(&mut ctx, 3);
}

#[test]
fn test_close_points_account_destination_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClosePointsAccountFixture>(&mut ctx, 4);
}

#[test]
fn test_close_points_account_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<ClosePointsAccountFixture>(&mut ctx);
}

#[test]
fn test_close_points_account_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<ClosePointsAccountFixture>(&mut ctx);
}

// ── Success tests ───────────────────────────────────────────────────────────

#[test]
fn test_close_points_account_success() {
    let mut ctx = TestContext::new();
    let setup = ClosePointsAccountSetup::new(&mut ctx);
    let ix = setup.build_instruction(&ctx);
    ix.send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &setup.user_points_pda);
}

// ── Error tests ─────────────────────────────────────────────────────────────

#[test]
fn test_close_points_account_balance_not_zero() {
    let mut ctx = TestContext::new();

    // Create config and issue points but DON'T use them
    let init_setup = InitPointsSetup::builder(&mut ctx).build();
    init_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let user = ctx.create_funded_keypair();
    let (user_points_pda, user_points_bump) = find_user_points_pda(&init_setup.points_config_pda, &user.pubkey());

    let issue = IssuePointsSetup {
        authority: init_setup.authority.insecure_clone(),
        points_config_pda: init_setup.points_config_pda,
        user: user.pubkey(),
        user_points_pda,
        user_points_bump,
        quantity: 100,
    };
    issue.build_instruction(&ctx).send_expect_success(&mut ctx);

    // Try to close account with non-zero balance
    let (event_authority, _) = find_event_authority_pda();
    let mut builder = rewards_program_client::instructions::ClosePointsAccountBuilder::new();
    builder
        .authority(init_setup.authority.pubkey())
        .points_config(init_setup.points_config_pda)
        .user(user.pubkey())
        .user_points_account(user_points_pda)
        .destination(ctx.payer.pubkey())
        .event_authority(event_authority);

    let ix = TestInstruction {
        instruction: builder.instruction(),
        signers: vec![init_setup.authority.insecure_clone()],
        name: "ClosePointsAccount",
    };
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::PointsBalanceNotZero);
}

#[test]
fn test_close_points_account_wrong_authority() {
    let mut ctx = TestContext::new();
    let setup = ClosePointsAccountSetup::new(&mut ctx);

    let fake_authority = ctx.create_funded_keypair();
    let bad_setup = ClosePointsAccountSetup {
        authority: fake_authority,
        user: setup.user,
        points_config_pda: setup.points_config_pda,
        user_points_pda: setup.user_points_pda,
        destination: setup.destination,
    };
    let ix = bad_setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}
