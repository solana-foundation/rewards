use solana_sdk::instruction::InstructionError;
use solana_sdk::signer::Signer;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;

use crate::fixtures::{ClosePointsAccountFixture, ClosePointsAccountSetup, InitPointsSetup, IssuePointsSetup};
use crate::utils::{
    assert_instruction_error, assert_rewards_error, assert_user_points_balance, find_event_authority_pda,
    test_empty_data, test_missing_signer, test_wrong_current_program, RewardsError, TestContext, TestInstruction,
};

// ── Generic validation tests ────────────────────────────────────────────────

#[test]
fn test_close_points_account_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<ClosePointsAccountFixture>(&mut ctx, 0, 0);
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
}

// ── Error tests ─────────────────────────────────────────────────────────────

#[test]
fn test_close_points_account_balance_not_zero() {
    let mut ctx = TestContext::new();

    // Create config and issue points but DON'T use them
    let init_setup = InitPointsSetup::builder(&mut ctx).build();
    init_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let user = ctx.create_funded_keypair();
    let user_ata = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &init_setup.points_mint_pda,
        &TOKEN_2022_PROGRAM_ID,
    );

    let issue = IssuePointsSetup {
        authority: init_setup.authority.insecure_clone(),
        points_config_pda: init_setup.points_config_pda,
        points_mint_pda: init_setup.points_mint_pda,
        user: user.pubkey(),
        user_ata,
        quantity: 100,
    };
    issue.build_instruction(&ctx).send_expect_success(&mut ctx);

    // Try to close account with non-zero balance
    let (event_authority, _) = find_event_authority_pda();
    let mut builder = rewards_program_client::instructions::ClosePointsAccountBuilder::new();
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
        points_mint_pda: setup.points_mint_pda,
        user_ata: setup.user_ata,
    };
    let ix = bad_setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}

#[test]
fn test_close_points_account_nonexistent_ata() {
    let mut ctx = TestContext::new();

    // Create config but do NOT issue any points (ATA never created)
    let init_setup = InitPointsSetup::builder(&mut ctx).build();
    init_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let user = ctx.create_funded_keypair();
    let user_ata = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &init_setup.points_mint_pda,
        &TOKEN_2022_PROGRAM_ID,
    );

    // Try to close an account that was never created
    let (event_authority, _) = find_event_authority_pda();
    let mut builder = rewards_program_client::instructions::ClosePointsAccountBuilder::new();
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
        name: "ClosePointsAccount",
    };
    let error = ix.send_expect_error(&mut ctx);
    assert_instruction_error(error, InstructionError::InvalidAccountOwner);
}

#[test]
fn test_close_points_account_then_reissue() {
    let mut ctx = TestContext::new();

    // Create config, issue, use all, close account
    let setup = ClosePointsAccountSetup::new(&mut ctx);
    let ix = setup.build_instruction(&ctx);
    ix.send_expect_success(&mut ctx);

    // Re-issue points to the same user (ATA creation is idempotent)
    let reissue = IssuePointsSetup {
        authority: setup.authority.insecure_clone(),
        points_config_pda: setup.points_config_pda,
        points_mint_pda: setup.points_mint_pda,
        user: setup.user.pubkey(),
        user_ata: setup.user_ata,
        quantity: 200,
    };
    reissue.build_instruction(&ctx).send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &setup.user.pubkey(), &setup.points_mint_pda, 200);
}
