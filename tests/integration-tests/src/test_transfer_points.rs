use solana_sdk::instruction::InstructionError;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;

use crate::fixtures::{InitPointsSetup, IssuePointsSetup, TransferPointsFixture, TransferPointsSetup};
use crate::utils::{
    assert_instruction_error, assert_rewards_error, assert_user_points_balance, test_empty_data, test_missing_signer,
    test_not_writable, test_truncated_data, test_wrong_current_program, test_wrong_system_program, RewardsError,
    TestContext,
};

// ── Generic validation tests ────────────────────────────────────────────────

#[test]
fn test_transfer_points_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<TransferPointsFixture>(&mut ctx, 1, 0);
}

#[test]
fn test_transfer_points_missing_from_user_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<TransferPointsFixture>(&mut ctx, 2, 1);
}

#[test]
fn test_transfer_points_from_token_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<TransferPointsFixture>(&mut ctx, 6);
}

#[test]
fn test_transfer_points_to_token_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<TransferPointsFixture>(&mut ctx, 7);
}

#[test]
fn test_transfer_points_wrong_system_program() {
    let mut ctx = TestContext::new();
    test_wrong_system_program::<TransferPointsFixture>(&mut ctx);
}

#[test]
fn test_transfer_points_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<TransferPointsFixture>(&mut ctx);
}

#[test]
fn test_transfer_points_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<TransferPointsFixture>(&mut ctx);
}

#[test]
fn test_transfer_points_truncated_data() {
    let mut ctx = TestContext::new();
    test_truncated_data::<TransferPointsFixture>(&mut ctx);
}

// ── Success tests ───────────────────────────────────────────────────────────

#[test]
fn test_transfer_points_success_to_new_user() {
    let mut ctx = TestContext::new();
    let setup = TransferPointsSetup::builder(&mut ctx).issue_quantity(1_000).quantity(400).build();
    let ix = setup.build_instruction(&ctx);
    ix.send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &setup.from_user.pubkey(), &setup.points_mint_pda, 600);
    assert_user_points_balance(&ctx, &setup.to_user, &setup.points_mint_pda, 400);
}

#[test]
fn test_transfer_points_success_full_balance() {
    let mut ctx = TestContext::new();
    let setup = TransferPointsSetup::builder(&mut ctx).issue_quantity(1_000).quantity(1_000).build();
    let ix = setup.build_instruction(&ctx);
    ix.send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &setup.from_user.pubkey(), &setup.points_mint_pda, 0);
    assert_user_points_balance(&ctx, &setup.to_user, &setup.points_mint_pda, 1_000);
}

// ── Error tests ─────────────────────────────────────────────────────────────

#[test]
fn test_transfer_points_zero_quantity() {
    let mut ctx = TestContext::new();
    let mut setup = TransferPointsSetup::builder(&mut ctx).issue_quantity(1_000).quantity(100).build();
    setup.quantity = 0;
    let ix = setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::InvalidAmount);
}

#[test]
fn test_transfer_points_disabled() {
    let mut ctx = TestContext::new();

    // Create config with transferable=0
    let init_setup = InitPointsSetup::builder(&mut ctx).transferable(0).build();
    let init_ix = init_setup.build_instruction(&ctx);
    init_ix.send_expect_success(&mut ctx);

    let from_user = ctx.create_funded_keypair();
    let from_token_account = get_associated_token_address_with_program_id(
        &from_user.pubkey(),
        &init_setup.points_mint_pda,
        &TOKEN_2022_PROGRAM_ID,
    );

    // Issue to from_user
    let issue = IssuePointsSetup {
        authority: init_setup.authority.insecure_clone(),
        points_config_pda: init_setup.points_config_pda,
        points_mint_pda: init_setup.points_mint_pda,
        user: from_user.pubkey(),
        user_ata: from_token_account,
        quantity: 1_000,
    };
    issue.build_instruction(&ctx).send_expect_success(&mut ctx);

    let to_user = solana_sdk::signature::Keypair::new();
    let to_token_account = get_associated_token_address_with_program_id(
        &to_user.pubkey(),
        &init_setup.points_mint_pda,
        &TOKEN_2022_PROGRAM_ID,
    );

    // Attempt transfer on non-transferable config
    let transfer_setup = TransferPointsSetup {
        authority: init_setup.authority,
        from_user,
        to_user: to_user.pubkey(),
        points_config_pda: init_setup.points_config_pda,
        points_mint_pda: init_setup.points_mint_pda,
        from_token_account,
        to_token_account,
        quantity: 500,
        issued_quantity: 1_000,
    };
    let ix = transfer_setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::PointsTransfersDisabled);
}

#[test]
fn test_transfer_points_insufficient_balance() {
    let mut ctx = TestContext::new();
    let setup = TransferPointsSetup::builder(&mut ctx).issue_quantity(100).quantity(200).build();
    let ix = setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::InsufficientPointsBalance);
}

#[test]
fn test_transfer_points_wrong_authority() {
    let mut ctx = TestContext::new();
    let setup = TransferPointsSetup::new(&mut ctx);

    let fake_authority = ctx.create_funded_keypair();
    let bad_setup = TransferPointsSetup {
        authority: fake_authority,
        from_user: setup.from_user,
        to_user: setup.to_user,
        points_config_pda: setup.points_config_pda,
        points_mint_pda: setup.points_mint_pda,
        from_token_account: setup.from_token_account,
        to_token_account: setup.to_token_account,
        quantity: setup.quantity,
        issued_quantity: setup.issued_quantity,
    };
    let ix = bad_setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}

#[test]
fn test_transfer_points_self_transfer() {
    let mut ctx = TestContext::new();
    let setup = TransferPointsSetup::new(&mut ctx);

    // Attempt transfer where from_user == to_user
    let self_transfer = TransferPointsSetup {
        authority: setup.authority,
        from_user: setup.from_user.insecure_clone(),
        to_user: setup.from_user.pubkey(),
        points_config_pda: setup.points_config_pda,
        points_mint_pda: setup.points_mint_pda,
        from_token_account: setup.from_token_account,
        to_token_account: setup.from_token_account,
        quantity: setup.quantity,
        issued_quantity: setup.issued_quantity,
    };
    let ix = self_transfer.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::PointsSelfTransferNotAllowed);
}

#[test]
fn test_transfer_points_wrong_from_ata() {
    let mut ctx = TestContext::new();
    let setup = TransferPointsSetup::new(&mut ctx);

    // Replace from_token_account with a fabricated address
    let wrong_ata = Keypair::new().pubkey();
    let bad_setup = TransferPointsSetup {
        authority: setup.authority,
        from_user: setup.from_user,
        to_user: setup.to_user,
        points_config_pda: setup.points_config_pda,
        points_mint_pda: setup.points_mint_pda,
        from_token_account: wrong_ata,
        to_token_account: setup.to_token_account,
        quantity: setup.quantity,
        issued_quantity: setup.issued_quantity,
    };
    let ix = bad_setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_instruction_error(error, InstructionError::InvalidAccountData);
}

#[test]
fn test_transfer_points_wrong_to_ata() {
    let mut ctx = TestContext::new();
    let setup = TransferPointsSetup::new(&mut ctx);

    // Replace to_token_account with a fabricated address
    let wrong_ata = Keypair::new().pubkey();
    let bad_setup = TransferPointsSetup {
        authority: setup.authority,
        from_user: setup.from_user,
        to_user: setup.to_user,
        points_config_pda: setup.points_config_pda,
        points_mint_pda: setup.points_mint_pda,
        from_token_account: setup.from_token_account,
        to_token_account: wrong_ata,
        quantity: setup.quantity,
        issued_quantity: setup.issued_quantity,
    };
    let ix = bad_setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_instruction_error(error, InstructionError::InvalidAccountData);
}

#[test]
fn test_transfer_points_to_existing_user() {
    let mut ctx = TestContext::new();

    // Create config with transferable=1
    let init_setup = InitPointsSetup::builder(&mut ctx).transferable(1).build();
    init_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let from_user = ctx.create_funded_keypair();
    let from_ata = get_associated_token_address_with_program_id(
        &from_user.pubkey(),
        &init_setup.points_mint_pda,
        &TOKEN_2022_PROGRAM_ID,
    );

    let to_user = solana_sdk::signature::Keypair::new();
    let to_ata = get_associated_token_address_with_program_id(
        &to_user.pubkey(),
        &init_setup.points_mint_pda,
        &TOKEN_2022_PROGRAM_ID,
    );

    // Issue 1000 to sender
    let issue_from = IssuePointsSetup {
        authority: init_setup.authority.insecure_clone(),
        points_config_pda: init_setup.points_config_pda,
        points_mint_pda: init_setup.points_mint_pda,
        user: from_user.pubkey(),
        user_ata: from_ata,
        quantity: 1_000,
    };
    issue_from.build_instruction(&ctx).send_expect_success(&mut ctx);

    // Issue 200 to recipient (pre-existing balance)
    let issue_to = IssuePointsSetup {
        authority: init_setup.authority.insecure_clone(),
        points_config_pda: init_setup.points_config_pda,
        points_mint_pda: init_setup.points_mint_pda,
        user: to_user.pubkey(),
        user_ata: to_ata,
        quantity: 200,
    };
    issue_to.build_instruction(&ctx).send_expect_success(&mut ctx);

    // Transfer 300 from sender to recipient (who already has 200)
    let transfer_setup = TransferPointsSetup {
        authority: init_setup.authority,
        from_user,
        to_user: to_user.pubkey(),
        points_config_pda: init_setup.points_config_pda,
        points_mint_pda: init_setup.points_mint_pda,
        from_token_account: from_ata,
        to_token_account: to_ata,
        quantity: 300,
        issued_quantity: 1_000,
    };
    let ix = transfer_setup.build_instruction(&ctx);
    ix.send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &transfer_setup.from_user.pubkey(), &init_setup.points_mint_pda, 700);
    assert_user_points_balance(&ctx, &transfer_setup.to_user, &init_setup.points_mint_pda, 500);
}

#[test]
fn test_transfer_points_chained_partial() {
    let mut ctx = TestContext::new();

    // Create config with transferable=1
    let init_setup = InitPointsSetup::builder(&mut ctx).transferable(1).build();
    init_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let from_user = ctx.create_funded_keypair();
    let from_ata = get_associated_token_address_with_program_id(
        &from_user.pubkey(),
        &init_setup.points_mint_pda,
        &TOKEN_2022_PROGRAM_ID,
    );

    let to_user = solana_sdk::signature::Keypair::new();
    let to_ata = get_associated_token_address_with_program_id(
        &to_user.pubkey(),
        &init_setup.points_mint_pda,
        &TOKEN_2022_PROGRAM_ID,
    );

    // Issue 1000 to sender
    let issue = IssuePointsSetup {
        authority: init_setup.authority.insecure_clone(),
        points_config_pda: init_setup.points_config_pda,
        points_mint_pda: init_setup.points_mint_pda,
        user: from_user.pubkey(),
        user_ata: from_ata,
        quantity: 1_000,
    };
    issue.build_instruction(&ctx).send_expect_success(&mut ctx);

    // Transfer 400
    let transfer1 = TransferPointsSetup {
        authority: init_setup.authority.insecure_clone(),
        from_user: from_user.insecure_clone(),
        to_user: to_user.pubkey(),
        points_config_pda: init_setup.points_config_pda,
        points_mint_pda: init_setup.points_mint_pda,
        from_token_account: from_ata,
        to_token_account: to_ata,
        quantity: 400,
        issued_quantity: 1_000,
    };
    transfer1.build_instruction(&ctx).send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &from_user.pubkey(), &init_setup.points_mint_pda, 600);
    assert_user_points_balance(&ctx, &to_user.pubkey(), &init_setup.points_mint_pda, 400);

    // Transfer remaining 600 (drains sender)
    ctx.advance_slot();
    let transfer2 = TransferPointsSetup {
        authority: init_setup.authority,
        from_user,
        to_user: to_user.pubkey(),
        points_config_pda: init_setup.points_config_pda,
        points_mint_pda: init_setup.points_mint_pda,
        from_token_account: from_ata,
        to_token_account: to_ata,
        quantity: 600,
        issued_quantity: 1_000,
    };
    transfer2.build_instruction(&ctx).send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &transfer2.from_user.pubkey(), &init_setup.points_mint_pda, 0);
    assert_user_points_balance(&ctx, &transfer2.to_user, &init_setup.points_mint_pda, 1_000);
}
