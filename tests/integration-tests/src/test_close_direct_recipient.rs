use rewards_program_client::instructions::AddDirectRecipientBuilder;
use rewards_program_client::types::VestingSchedule;
use solana_sdk::{instruction::InstructionError, signature::Signer};

use crate::fixtures::{
    AddDirectRecipientSetup, ClaimDirectSetup, CloseDirectDistributionSetup, CloseDirectRecipientFixture,
    CloseDirectRecipientSetup, CreateDirectDistributionSetup,
};
use crate::utils::{
    assert_account_closed, assert_instruction_error, assert_rewards_error, find_direct_recipient_pda,
    find_event_authority_pda, test_empty_data, test_not_writable, test_wrong_current_program, RewardsError,
    TestContext, TestInstruction,
};

#[test]
fn test_close_direct_recipient_permissionless_when_fully_claimed() {
    let mut ctx = TestContext::new();
    let setup = CloseDirectRecipientSetup::new(&mut ctx);

    // Build instruction — no signer needed since recipient is fully claimed
    let test_ix = setup.build_instruction(&ctx);
    assert!(test_ix.signers.is_empty(), "No signers should be required");
    test_ix.send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &setup.recipient_pda);
}

#[test]
fn test_close_direct_recipient_original_payer_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CloseDirectRecipientFixture>(&mut ctx, 1);
}

#[test]
fn test_close_direct_recipient_recipient_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CloseDirectRecipientFixture>(&mut ctx, 3);
}

#[test]
fn test_close_direct_recipient_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<CloseDirectRecipientFixture>(&mut ctx);
}

#[test]
fn test_close_direct_recipient_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<CloseDirectRecipientFixture>(&mut ctx);
}

#[test]
fn test_close_direct_recipient_success() {
    let mut ctx = TestContext::new();
    let setup = CloseDirectRecipientSetup::new(&mut ctx);

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &setup.recipient_pda);
}

#[test]
fn test_close_direct_recipient_success_token_2022() {
    let mut ctx = TestContext::new();
    let setup = CloseDirectRecipientSetup::new_token_2022(&mut ctx);

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &setup.recipient_pda);
}

#[test]
fn test_close_direct_recipient_after_distribution_closed() {
    let mut ctx = TestContext::new();

    let recipient_setup = AddDirectRecipientSetup::new(&mut ctx);
    let add_ix = recipient_setup.build_instruction(&ctx);
    add_ix.send_expect_success(&mut ctx);

    let close_distribution_setup = CloseDirectDistributionSetup {
        authority: recipient_setup.authority.insecure_clone(),
        distribution_pda: recipient_setup.distribution_pda,
        mint: recipient_setup.mint,
        distribution_vault: recipient_setup.distribution_vault,
        authority_token_account: recipient_setup.authority_token_account,
        token_program: recipient_setup.token_program,
    };

    let close_distribution_ix = close_distribution_setup.build_instruction(&ctx);
    close_distribution_ix.send_expect_success(&mut ctx);
    // After close, the distribution PDA still lives on as a DirectDistributionClosed marker (3 bytes, disc=8).
    let closed_account =
        ctx.get_account(&recipient_setup.distribution_pda).expect("Distribution PDA should still exist after close");
    assert_eq!(closed_account.data.len(), 3, "Closed distribution should be 3 bytes");
    assert_eq!(closed_account.data[0], 8, "First byte should be DirectDistributionClosed discriminator");

    let close_recipient_setup = CloseDirectRecipientSetup {
        recipient: recipient_setup.recipient.insecure_clone(),
        original_payer: ctx.payer.insecure_clone(),
        distribution_pda: recipient_setup.distribution_pda,
        recipient_pda: recipient_setup.recipient_pda,
        token_program: recipient_setup.token_program,
    };

    let close_recipient_ix = close_recipient_setup.build_instruction(&ctx);
    close_recipient_ix.send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &recipient_setup.recipient_pda);
}

#[test]
fn test_close_direct_recipient_claim_not_fully_vested() {
    let mut ctx = TestContext::new();

    let amount = 1_000_000u64;
    let distribution_setup = CreateDirectDistributionSetup::new(&mut ctx);
    let create_ix = distribution_setup.build_instruction(&ctx);
    create_ix.send_expect_success(&mut ctx);

    let current_ts = ctx.get_current_timestamp();
    let recipient = ctx.create_funded_keypair();
    let rent_payer = ctx.create_funded_keypair();
    let (recipient_pda, recipient_bump) =
        find_direct_recipient_pda(&distribution_setup.distribution_pda, &recipient.pubkey());

    let authority_token_account = ctx.create_ata_for_program_with_balance(
        &distribution_setup.authority.pubkey(),
        &distribution_setup.mint.pubkey(),
        amount,
        &distribution_setup.token_program,
    );

    let (event_authority, _) = find_event_authority_pda();
    let mut add_builder = AddDirectRecipientBuilder::new();
    add_builder
        .payer(rent_payer.pubkey())
        .authority(distribution_setup.authority.pubkey())
        .distribution(distribution_setup.distribution_pda)
        .recipient_account(recipient_pda)
        .recipient(recipient.pubkey())
        .mint(distribution_setup.mint.pubkey())
        .distribution_vault(distribution_setup.distribution_vault)
        .authority_token_account(authority_token_account)
        .token_program(distribution_setup.token_program)
        .event_authority(event_authority)
        .bump(recipient_bump)
        .amount(amount)
        .schedule(VestingSchedule::Linear { start_ts: current_ts, end_ts: current_ts + 86400 * 365 });

    let add_ix = TestInstruction {
        instruction: add_builder.instruction(),
        signers: vec![rent_payer.insecure_clone(), distribution_setup.authority.insecure_clone()],
        name: "AddDirectRecipient",
    };
    add_ix.send_expect_success(&mut ctx);

    // Warp to 50% and claim partial
    let mid_point = current_ts + (86400 * 365) / 2;
    ctx.warp_to_timestamp(mid_point);

    let recipient_token_account = ctx.create_token_account(&recipient.pubkey(), &distribution_setup.mint.pubkey());

    let claim_setup = ClaimDirectSetup {
        recipient: recipient.insecure_clone(),
        distribution_pda: distribution_setup.distribution_pda,
        recipient_pda,
        recipient_bump,
        mint: distribution_setup.mint.pubkey(),
        distribution_vault: distribution_setup.distribution_vault,
        recipient_token_account,
        token_program: distribution_setup.token_program,
        amount,
        start_ts: current_ts,
        end_ts: current_ts + 86400 * 365,
    };
    let claim_ix = claim_setup.build_instruction(&ctx);
    claim_ix.send_expect_success(&mut ctx);

    // Try to close — should fail because only ~50% claimed
    let close_setup = CloseDirectRecipientSetup {
        recipient: recipient.insecure_clone(),
        original_payer: rent_payer,
        distribution_pda: distribution_setup.distribution_pda,
        recipient_pda,
        token_program: distribution_setup.token_program,
    };

    let test_ix = close_setup.build_instruction(&ctx);
    let error = test_ix.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::ClaimNotFullyVested);
}

#[test]
fn test_close_direct_recipient_wrong_original_payer() {
    let mut ctx = TestContext::new();
    let setup = CloseDirectRecipientSetup::new(&mut ctx);

    let wrong_payer = ctx.create_funded_keypair();

    let test_ix = setup.build_instruction_with_wrong_original_payer(&ctx, wrong_payer.pubkey());
    let error = test_ix.send_expect_error(&mut ctx);

    assert_instruction_error(error, InstructionError::InvalidAccountData);
}

#[test]
fn test_close_direct_recipient_wrong_recipient() {
    let mut ctx = TestContext::new();
    let setup = CloseDirectRecipientSetup::new(&mut ctx);

    let wrong_recipient = ctx.create_funded_keypair();

    let test_ix = setup.build_instruction_with_wrong_recipient(&ctx, &wrong_recipient);
    let error = test_ix.send_expect_error(&mut ctx);

    // The wrong recipient's PDA doesn't exist (owned by system program)
    assert_instruction_error(error, InstructionError::InvalidAccountOwner);
}

#[test]
fn test_close_direct_recipient_returns_rent() {
    let mut ctx = TestContext::new();
    let setup = CloseDirectRecipientSetup::new(&mut ctx);

    let payer_sol_before = ctx.get_account(&setup.original_payer.pubkey()).map(|a| a.lamports).unwrap_or(0);

    let recipient_account = ctx.get_account(&setup.recipient_pda).expect("Recipient should exist");
    let recipient_rent = recipient_account.lamports;

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let payer_sol_after = ctx.get_account(&setup.original_payer.pubkey()).map(|a| a.lamports).unwrap_or(0);

    assert_eq!(
        payer_sol_after,
        payer_sol_before + recipient_rent,
        "Original payer should receive exact rent lamports back"
    );
}
