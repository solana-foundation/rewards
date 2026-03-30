use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;

use crate::fixtures::{InitPointsSetup, IssuePointsSetup, RevokePointsFixture, RevokePointsSetup};
use crate::utils::{
    assert_rewards_error, assert_user_points_balance, find_event_authority_pda, test_empty_data, test_missing_signer,
    test_not_writable, test_wrong_current_program, RewardsError, TestContext, TestInstruction,
};

// ── Generic validation tests ────────────────────────────────────────────────

#[test]
fn test_revoke_points_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<RevokePointsFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_revoke_points_points_mint_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<RevokePointsFixture>(&mut ctx, 2);
}

#[test]
fn test_revoke_points_user_token_account_not_writable() {
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

    // Tokens should be burned — balance is zero but account still exists
    assert_user_points_balance(&ctx, &setup.user, &setup.points_mint_pda, 0);
}

// ── Error tests ─────────────────────────────────────────────────────────────

#[test]
fn test_revoke_points_not_revocable() {
    let mut ctx = TestContext::new();

    // Create config with revocable=0
    let init_setup = InitPointsSetup::builder(&mut ctx).revocable(0).build();
    init_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let user = Keypair::new();
    let user_ata = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &init_setup.points_mint_pda,
        &TOKEN_2022_PROGRAM_ID,
    );

    // Issue points
    let issue = IssuePointsSetup {
        authority: init_setup.authority.insecure_clone(),
        points_config_pda: init_setup.points_config_pda,
        points_mint_pda: init_setup.points_mint_pda,
        user: user.pubkey(),
        user_ata,
        quantity: 100,
    };
    issue.build_instruction(&ctx).send_expect_success(&mut ctx);

    // Attempt revoke on non-revocable config
    let (event_authority, _) = find_event_authority_pda();
    let mut builder = rewards_program_client::instructions::RevokePointsBuilder::new();
    builder
        .authority(init_setup.authority.pubkey())
        .points_config(init_setup.points_config_pda)
        .points_mint(init_setup.points_mint_pda)
        .user(user.pubkey())
        .user_token_account(user_ata)
        .token2022_program(TOKEN_2022_PROGRAM_ID)
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
        points_mint_pda: setup.points_mint_pda,
        user_ata: setup.user_ata,
        issued_quantity: setup.issued_quantity,
    };
    let ix = bad_setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}

#[test]
fn test_revoke_points_zero_balance() {
    let mut ctx = TestContext::new();

    // Create revocable config and issue points
    let setup = RevokePointsSetup::new_with_quantity(&mut ctx, 500);

    // Revoke all points first
    let ix1 = setup.build_instruction(&ctx);
    ix1.send_expect_success(&mut ctx);
    assert_user_points_balance(&ctx, &setup.user, &setup.points_mint_pda, 0);

    // Revoke again on zero-balance account — should error
    ctx.advance_slot();
    let ix2 = setup.build_instruction(&ctx);
    let error = ix2.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::PointsNothingToRevoke);
}

#[test]
fn test_revoke_points_then_reissue() {
    let mut ctx = TestContext::new();

    // Create revocable config and issue points
    let init_setup = crate::fixtures::InitPointsSetup::builder(&mut ctx).revocable(1).build();
    init_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let user = Keypair::new();
    let user_ata = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &init_setup.points_mint_pda,
        &TOKEN_2022_PROGRAM_ID,
    );

    // Issue 500 points
    let issue = IssuePointsSetup {
        authority: init_setup.authority.insecure_clone(),
        points_config_pda: init_setup.points_config_pda,
        points_mint_pda: init_setup.points_mint_pda,
        user: user.pubkey(),
        user_ata,
        quantity: 500,
    };
    issue.build_instruction(&ctx).send_expect_success(&mut ctx);
    assert_user_points_balance(&ctx, &user.pubkey(), &init_setup.points_mint_pda, 500);

    // Revoke all
    let revoke = RevokePointsSetup {
        authority: init_setup.authority.insecure_clone(),
        user: user.pubkey(),
        points_config_pda: init_setup.points_config_pda,
        points_mint_pda: init_setup.points_mint_pda,
        user_ata,
        issued_quantity: 500,
    };
    revoke.build_instruction(&ctx).send_expect_success(&mut ctx);
    assert_user_points_balance(&ctx, &user.pubkey(), &init_setup.points_mint_pda, 0);

    // Re-issue 300 points to same user
    let reissue = IssuePointsSetup {
        authority: init_setup.authority.insecure_clone(),
        points_config_pda: init_setup.points_config_pda,
        points_mint_pda: init_setup.points_mint_pda,
        user: user.pubkey(),
        user_ata,
        quantity: 300,
    };
    reissue.build_instruction(&ctx).send_expect_success(&mut ctx);
    assert_user_points_balance(&ctx, &user.pubkey(), &init_setup.points_mint_pda, 300);
}
