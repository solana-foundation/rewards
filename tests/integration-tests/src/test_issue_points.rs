use solana_sdk::signature::{Keypair, Signer};

use crate::fixtures::{IssuePointsFixture, IssuePointsSetup};
use crate::utils::{
    assert_rewards_error, assert_user_points_balance, find_user_points_pda, get_points_config, test_empty_data,
    test_missing_signer, test_not_writable, test_truncated_data, test_wrong_current_program, test_wrong_system_program,
    RewardsError, TestContext,
};

// ── Generic validation tests ────────────────────────────────────────────────

#[test]
fn test_issue_points_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<IssuePointsFixture>(&mut ctx, 1, 0);
}

#[test]
fn test_issue_points_config_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<IssuePointsFixture>(&mut ctx, 2);
}

#[test]
fn test_issue_points_user_points_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<IssuePointsFixture>(&mut ctx, 4);
}

#[test]
fn test_issue_points_wrong_system_program() {
    let mut ctx = TestContext::new();
    test_wrong_system_program::<IssuePointsFixture>(&mut ctx);
}

#[test]
fn test_issue_points_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<IssuePointsFixture>(&mut ctx);
}

#[test]
fn test_issue_points_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<IssuePointsFixture>(&mut ctx);
}

#[test]
fn test_issue_points_truncated_data() {
    let mut ctx = TestContext::new();
    test_truncated_data::<IssuePointsFixture>(&mut ctx);
}

// ── Success tests ───────────────────────────────────────────────────────────

#[test]
fn test_issue_points_success_new_user() {
    let mut ctx = TestContext::new();
    let setup = IssuePointsSetup::new(&mut ctx);
    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &setup.user_points_pda, setup.quantity);

    let config = get_points_config(&ctx, &setup.points_config_pda);
    assert_eq!(config.total_issued, setup.quantity);
    assert_eq!(config.total_used, 0);
}

#[test]
fn test_issue_points_success_existing_user_increments() {
    let mut ctx = TestContext::new();
    let setup = IssuePointsSetup::builder(&mut ctx).quantity(500).build();

    // First issue
    let ix1 = setup.build_instruction(&ctx);
    ix1.send_expect_success(&mut ctx);
    assert_user_points_balance(&ctx, &setup.user_points_pda, 500);

    // Second issue to same user
    let setup2 = IssuePointsSetup {
        authority: setup.authority.insecure_clone(),
        points_config_pda: setup.points_config_pda,
        user: setup.user,
        user_points_pda: setup.user_points_pda,
        user_points_bump: setup.user_points_bump,
        quantity: 300,
    };
    let ix2 = setup2.build_instruction(&ctx);
    ix2.send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &setup.user_points_pda, 800);

    let config = crate::utils::get_points_config(&ctx, &setup.points_config_pda);
    assert_eq!(config.total_issued, 800);
}

// ── Error tests ─────────────────────────────────────────────────────────────

#[test]
fn test_issue_points_zero_quantity() {
    let mut ctx = TestContext::new();
    let mut setup = IssuePointsSetup::builder(&mut ctx).build();
    setup.quantity = 0;
    let ix = setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::InvalidAmount);
}

#[test]
fn test_issue_points_max_supply_exceeded() {
    let mut ctx = TestContext::new();
    let setup = IssuePointsSetup::builder(&mut ctx).max_supply(500).quantity(501).build();
    let ix = setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::PointsMaxSupplyExceeded);
}

#[test]
fn test_issue_points_max_supply_exceeded_cumulative() {
    let mut ctx = TestContext::new();
    let setup = IssuePointsSetup::builder(&mut ctx).max_supply(1000).quantity(600).build();

    // First issue of 600 succeeds
    let ix1 = setup.build_instruction(&ctx);
    ix1.send_expect_success(&mut ctx);

    // Second issue of 600 to another user should fail (total would be 1200 > 1000)
    let user2 = Keypair::new();
    let (user2_points_pda, user2_points_bump) = find_user_points_pda(&setup.points_config_pda, &user2.pubkey());
    let setup2 = IssuePointsSetup {
        authority: setup.authority.insecure_clone(),
        points_config_pda: setup.points_config_pda,
        user: user2.pubkey(),
        user_points_pda: user2_points_pda,
        user_points_bump: user2_points_bump,
        quantity: 600,
    };
    let ix2 = setup2.build_instruction(&ctx);
    let error = ix2.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::PointsMaxSupplyExceeded);
}

#[test]
fn test_issue_points_wrong_authority() {
    let mut ctx = TestContext::new();
    let setup = IssuePointsSetup::new(&mut ctx);

    // Replace authority with a different keypair
    let fake_authority = ctx.create_funded_keypair();
    let bad_setup = IssuePointsSetup {
        authority: fake_authority,
        points_config_pda: setup.points_config_pda,
        user: setup.user,
        user_points_pda: setup.user_points_pda,
        user_points_bump: setup.user_points_bump,
        quantity: setup.quantity,
    };
    let ix = bad_setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}
