use crate::fixtures::{IssuePointsFixture, IssuePointsSetup};
use crate::utils::{
    assert_rewards_error, assert_user_points_balance, test_empty_data, test_missing_signer, test_truncated_data,
    test_wrong_current_program, test_wrong_system_program, RewardsError, TestContext,
};

// ── Generic validation tests ────────────────────────────────────────────────

#[test]
fn test_issue_points_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<IssuePointsFixture>(&mut ctx, 1, 0);
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

    assert_user_points_balance(&ctx, &setup.user, &setup.points_mint_pda, setup.quantity);
}

#[test]
fn test_issue_points_success_existing_user_increments() {
    let mut ctx = TestContext::new();
    let setup = IssuePointsSetup::builder(&mut ctx).quantity(500).build();

    // First issue
    let ix1 = setup.build_instruction(&ctx);
    ix1.send_expect_success(&mut ctx);
    assert_user_points_balance(&ctx, &setup.user, &setup.points_mint_pda, 500);

    // Second issue to same user
    let setup2 = IssuePointsSetup {
        authority: setup.authority.insecure_clone(),
        points_config_pda: setup.points_config_pda,
        points_mint_pda: setup.points_mint_pda,
        user: setup.user,
        user_ata: setup.user_ata,
        quantity: 300,
    };
    let ix2 = setup2.build_instruction(&ctx);
    ix2.send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &setup.user, &setup.points_mint_pda, 800);
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
fn test_issue_points_wrong_authority() {
    let mut ctx = TestContext::new();
    let setup = IssuePointsSetup::new(&mut ctx);

    // Replace authority with a different keypair
    let fake_authority = ctx.create_funded_keypair();
    let bad_setup = IssuePointsSetup {
        authority: fake_authority,
        points_config_pda: setup.points_config_pda,
        points_mint_pda: setup.points_mint_pda,
        user: setup.user,
        user_ata: setup.user_ata,
        quantity: setup.quantity,
    };
    let ix = bad_setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}
