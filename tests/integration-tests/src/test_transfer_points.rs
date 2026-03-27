use solana_sdk::{instruction::InstructionError, signer::Signer};

use crate::fixtures::{InitPointsSetup, IssuePointsSetup, TransferPointsFixture, TransferPointsSetup};
use crate::utils::{
    assert_instruction_error, assert_rewards_error, assert_user_points_balance, find_user_points_pda,
    get_points_config, test_empty_data, test_missing_signer, test_not_writable, test_truncated_data,
    test_wrong_current_program, test_wrong_system_program, RewardsError, TestContext,
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
fn test_transfer_points_from_user_points_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<TransferPointsFixture>(&mut ctx, 4);
}

#[test]
fn test_transfer_points_to_user_points_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<TransferPointsFixture>(&mut ctx, 6);
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

    assert_user_points_balance(&ctx, &setup.from_user_points_pda, 600);
    assert_user_points_balance(&ctx, &setup.to_user_points_pda, 400);

    // Config totals should be unchanged by transfers
    let config = get_points_config(&ctx, &setup.points_config_pda);
    assert_eq!(config.total_issued, 1_000);
    assert_eq!(config.total_used, 0);
}

#[test]
fn test_transfer_points_success_full_balance() {
    let mut ctx = TestContext::new();
    let setup = TransferPointsSetup::builder(&mut ctx).issue_quantity(1_000).quantity(1_000).build();
    let ix = setup.build_instruction(&ctx);
    ix.send_expect_success(&mut ctx);

    assert_user_points_balance(&ctx, &setup.from_user_points_pda, 0);
    assert_user_points_balance(&ctx, &setup.to_user_points_pda, 1_000);
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
    let (from_pda, from_bump) = find_user_points_pda(&init_setup.points_config_pda, &from_user.pubkey());

    // Issue to from_user
    let issue = IssuePointsSetup {
        authority: init_setup.authority.insecure_clone(),
        points_config_pda: init_setup.points_config_pda,
        user: from_user.pubkey(),
        user_points_pda: from_pda,
        user_points_bump: from_bump,
        quantity: 1_000,
    };
    issue.build_instruction(&ctx).send_expect_success(&mut ctx);

    let to_user = solana_sdk::signature::Keypair::new();
    let (to_pda, to_bump) = find_user_points_pda(&init_setup.points_config_pda, &to_user.pubkey());

    // Attempt transfer on non-transferable config
    let transfer_setup = TransferPointsSetup {
        authority: init_setup.authority,
        from_user,
        to_user: to_user.pubkey(),
        points_config_pda: init_setup.points_config_pda,
        from_user_points_pda: from_pda,
        to_user_points_pda: to_pda,
        to_user_points_bump: to_bump,
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
        from_user_points_pda: setup.from_user_points_pda,
        to_user_points_pda: setup.to_user_points_pda,
        to_user_points_bump: setup.to_user_points_bump,
        quantity: setup.quantity,
        issued_quantity: setup.issued_quantity,
    };
    let ix = bad_setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}

#[test]
fn test_transfer_points_from_user_pda_mismatch() {
    let mut ctx = TestContext::new();
    let setup = TransferPointsSetup::new(&mut ctx);

    // Derive a PDA for a different user (not from_user)
    let other_user = solana_sdk::signature::Keypair::new();
    let (wrong_from_pda, _) = find_user_points_pda(&setup.points_config_pda, &other_user.pubkey());

    // Use correct from_user signer but wrong from_user_points PDA
    let bad_setup = TransferPointsSetup {
        authority: setup.authority,
        from_user: setup.from_user, // correct signer
        to_user: setup.to_user,
        points_config_pda: setup.points_config_pda,
        from_user_points_pda: wrong_from_pda, // doesn't derive from from_user
        to_user_points_pda: setup.to_user_points_pda,
        to_user_points_bump: setup.to_user_points_bump,
        quantity: setup.quantity,
        issued_quantity: setup.issued_quantity,
    };
    let ix = bad_setup.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    // PDA doesn't exist on-chain, so ownership check fires before PDA derivation
    assert_instruction_error(error, InstructionError::InvalidAccountOwner);
}

#[test]
fn test_transfer_points_self_transfer() {
    let mut ctx = TestContext::new();
    let setup = TransferPointsSetup::new(&mut ctx);

    // Derive the correct bump for from_user's points PDA
    let (_, from_bump) = find_user_points_pda(&setup.points_config_pda, &setup.from_user.pubkey());

    // Attempt transfer where from_user == to_user
    let self_transfer = TransferPointsSetup {
        authority: setup.authority,
        from_user: setup.from_user.insecure_clone(),
        to_user: setup.from_user.pubkey(),
        points_config_pda: setup.points_config_pda,
        from_user_points_pda: setup.from_user_points_pda,
        to_user_points_pda: setup.from_user_points_pda,
        to_user_points_bump: from_bump,
        quantity: setup.quantity,
        issued_quantity: setup.issued_quantity,
    };
    let ix = self_transfer.build_instruction(&ctx);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::PointsSelfTransferNotAllowed);
}
