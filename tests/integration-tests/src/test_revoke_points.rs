use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

use crate::fixtures::{InitPointsSetup, IssuePointsSetup, RevokePointsFixture, RevokePointsSetup};
use crate::utils::{
    assert_account_closed, assert_rewards_error, find_event_authority_pda, find_user_points_pda, get_points_config,
    test_empty_data, test_missing_signer, test_not_writable, test_wrong_current_program, RewardsError, TestContext,
    TestInstruction,
};

// ── Generic validation tests ────────────────────────────────────────────────

#[test]
fn test_revoke_points_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<RevokePointsFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_revoke_points_config_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<RevokePointsFixture>(&mut ctx, 1);
}

#[test]
fn test_revoke_points_user_points_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<RevokePointsFixture>(&mut ctx, 3);
}

#[test]
fn test_revoke_points_destination_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<RevokePointsFixture>(&mut ctx, 4);
}

#[test]
fn test_revoke_points_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<RevokePointsFixture>(&mut ctx);
}

#[test]
fn test_revoke_points_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<RevokePointsFixture>(&mut ctx);
}

// ── Success tests ───────────────────────────────────────────────────────────

#[test]
fn test_revoke_points_success() {
    let mut ctx = TestContext::new();
    let setup = RevokePointsSetup::new_with_quantity(&mut ctx, 750);
    let ix = setup.build_instruction(&ctx);
    ix.send_expect_success(&mut ctx);

    // User account should be closed
    assert_account_closed(&ctx, &setup.user_points_pda);

    // Config total_used should be incremented by the revoked balance
    let config = get_points_config(&ctx, &setup.points_config_pda);
    assert_eq!(config.total_used, 750);
    assert_eq!(config.total_issued, 750);
}

// ── Error tests ─────────────────────────────────────────────────────────────

#[test]
fn test_revoke_points_not_revocable() {
    let mut ctx = TestContext::new();

    // Create config with revocable=0
    let init_setup = InitPointsSetup::builder(&mut ctx).revocable(0).build();
    init_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let user = Keypair::new();
    let (user_points_pda, user_points_bump) = find_user_points_pda(&init_setup.points_config_pda, &user.pubkey());

    // Issue points
    let issue = IssuePointsSetup {
        authority: init_setup.authority.insecure_clone(),
        points_config_pda: init_setup.points_config_pda,
        user: user.pubkey(),
        user_points_pda,
        user_points_bump,
        quantity: 100,
    };
    issue.build_instruction(&ctx).send_expect_success(&mut ctx);

    // Attempt revoke on non-revocable config
    let (event_authority, _) = find_event_authority_pda();
    let mut builder = rewards_program_client::instructions::RevokePointsBuilder::new();
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
        name: "RevokePoints",
    };
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::PointsNotRevocable);
}

#[test]
fn test_revoke_points_wrong_authority() {
    let mut ctx = TestContext::new();
    let setup = RevokePointsSetup::new(&mut ctx);

    let fake_authority = ctx.create_funded_keypair();
    let bad_setup = RevokePointsSetup {
        authority: fake_authority,
        user: setup.user,
        points_config_pda: setup.points_config_pda,
        user_points_pda: setup.user_points_pda,
        destination: setup.destination,
        issued_quantity: setup.issued_quantity,
    };
    let ix = bad_setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}
