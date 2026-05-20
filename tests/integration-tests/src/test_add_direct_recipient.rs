use rewards_program_client::{instructions::CreateDirectDistributionBuilder, types::VestingSchedule};
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
};
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

use crate::fixtures::{
    AddDirectRecipientFixture, AddDirectRecipientSetup, CreateDirectDistributionSetup, DEFAULT_RECIPIENT_AMOUNT,
};
use crate::utils::{
    assert_direct_recipient, assert_instruction_error, assert_rewards_error, find_direct_distribution_pda,
    find_direct_recipient_pda, find_event_authority_pda, test_empty_data, test_missing_signer, test_not_writable,
    test_truncated_data, test_wrong_current_program, test_wrong_system_program, RewardsError, TestContext,
};

#[test]
fn test_add_direct_recipient_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<AddDirectRecipientFixture>(&mut ctx, 1, 0);
}

#[test]
fn test_add_direct_recipient_distribution_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<AddDirectRecipientFixture>(&mut ctx, 2);
}

#[test]
fn test_add_direct_recipient_recipient_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<AddDirectRecipientFixture>(&mut ctx, 3);
}

#[test]
fn test_add_direct_recipient_wrong_system_program() {
    let mut ctx = TestContext::new();
    test_wrong_system_program::<AddDirectRecipientFixture>(&mut ctx);
}

#[test]
fn test_add_direct_recipient_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<AddDirectRecipientFixture>(&mut ctx);
}

#[test]
fn test_add_direct_recipient_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<AddDirectRecipientFixture>(&mut ctx);
}

#[test]
fn test_add_direct_recipient_truncated_data() {
    let mut ctx = TestContext::new();
    test_truncated_data::<AddDirectRecipientFixture>(&mut ctx);
}

#[test]
fn test_add_direct_recipient_success() {
    let mut ctx = TestContext::new();
    let setup = AddDirectRecipientSetup::new(&mut ctx);
    let instruction = setup.build_instruction(&ctx);

    instruction.send_expect_success(&mut ctx);

    assert_direct_recipient(
        &ctx,
        &setup.recipient_pda,
        &setup.recipient.pubkey(),
        setup.amount,
        0,
        setup.recipient_bump,
    );
}

#[test]
fn test_add_direct_recipient_success_token_2022() {
    let mut ctx = TestContext::new();
    let setup = AddDirectRecipientSetup::new_token_2022(&mut ctx);
    let instruction = setup.build_instruction(&ctx);

    instruction.send_expect_success(&mut ctx);

    assert_direct_recipient(
        &ctx,
        &setup.recipient_pda,
        &setup.recipient.pubkey(),
        setup.amount,
        0,
        setup.recipient_bump,
    );
}

#[test]
fn test_add_direct_recipient_authority_can_add_self() {
    let mut ctx = TestContext::new();
    let seed = Keypair::new();
    let mint = Keypair::new();
    let token_program = TOKEN_PROGRAM_ID;

    ctx.create_mint_for_program(&mint, &ctx.payer.pubkey(), 6, &token_program);

    let (distribution_pda, distribution_bump) =
        find_direct_distribution_pda(&mint.pubkey(), &ctx.payer.pubkey(), &seed.pubkey());
    let distribution_vault = ctx.create_ata_for_program(&distribution_pda, &mint.pubkey(), &token_program);
    let (event_authority, _) = find_event_authority_pda();

    let mut create_builder = CreateDirectDistributionBuilder::new();
    create_builder
        .payer(ctx.payer.pubkey())
        .authority(ctx.payer.pubkey())
        .seeds(seed.pubkey())
        .distribution(distribution_pda)
        .mint(mint.pubkey())
        .distribution_vault(distribution_vault)
        .token_program(token_program)
        .event_authority(event_authority)
        .bump(distribution_bump)
        .revocable(0)
        .clawback_ts(0);

    ctx.send_transaction(create_builder.instruction(), &[&seed]).expect("distribution create should succeed");

    let recipient = ctx.payer.insecure_clone();
    let (recipient_pda, recipient_bump) = find_direct_recipient_pda(&distribution_pda, &recipient.pubkey());
    let authority_token_account = ctx.create_ata_for_program_with_balance(
        &ctx.payer.pubkey(),
        &mint.pubkey(),
        DEFAULT_RECIPIENT_AMOUNT,
        &token_program,
    );
    let current_ts = ctx.get_current_timestamp();

    let setup = AddDirectRecipientSetup {
        authority: ctx.payer.insecure_clone(),
        distribution_pda,
        recipient,
        recipient_pda,
        recipient_bump,
        amount: DEFAULT_RECIPIENT_AMOUNT,
        schedule: VestingSchedule::Linear { start_ts: current_ts, end_ts: current_ts + 86400 * 365 },
        token_program,
        mint: mint.pubkey(),
        distribution_vault,
        authority_token_account,
    };

    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    assert_direct_recipient(
        &ctx,
        &setup.recipient_pda,
        &setup.recipient.pubkey(),
        setup.amount,
        0,
        setup.recipient_bump,
    );
}

#[test]
fn test_add_direct_recipient_rejects_unrelated_writable_recipient() {
    let mut ctx = TestContext::new();
    let setup = AddDirectRecipientSetup::new(&mut ctx);
    let mut instruction = setup.build_instruction(&ctx);

    instruction.instruction.accounts[4].is_writable = true;
    let error = instruction.send_expect_error(&mut ctx);

    assert_instruction_error(error, InstructionError::AccountBorrowFailed);
}

#[test]
fn test_add_direct_recipient_unauthorized() {
    let mut ctx = TestContext::new();
    let setup = AddDirectRecipientSetup::new(&mut ctx);
    let wrong_authority = ctx.create_funded_keypair();

    let instruction = setup.build_instruction_with_wrong_authority(&ctx, &wrong_authority);
    let error = instruction.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}

#[test]
fn test_add_direct_recipient_zero_amount() {
    let mut ctx = TestContext::new();
    let setup = AddDirectRecipientSetup::builder(&mut ctx).amount(0).build();

    let instruction = setup.build_instruction(&ctx);
    let error = instruction.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::InvalidAmount);
}

#[test]
fn test_add_direct_recipient_invalid_time_window() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();

    let setup = AddDirectRecipientSetup::builder(&mut ctx)
        .schedule(VestingSchedule::Linear { start_ts: current_ts + 100, end_ts: current_ts + 50 })
        .build();

    let instruction = setup.build_instruction(&ctx);
    let error = instruction.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::InvalidTimeWindow);
}

#[test]
fn test_add_direct_recipient_multiple() {
    let mut ctx = TestContext::new();
    let distribution_setup = CreateDirectDistributionSetup::new(&mut ctx);
    let create_instruction = distribution_setup.build_instruction(&ctx);
    create_instruction.send_expect_success(&mut ctx);

    let current_ts = ctx.get_current_timestamp();
    let start_ts = current_ts;
    let end_ts = current_ts + 86400 * 365;

    let recipient1 = ctx.create_funded_keypair();
    let (recipient1_pda, recipient1_bump) =
        find_direct_recipient_pda(&distribution_setup.distribution_pda, &recipient1.pubkey());

    let schedule = VestingSchedule::Linear { start_ts, end_ts };

    let authority_token_account1 = ctx.create_ata_for_program_with_balance(
        &distribution_setup.authority.pubkey(),
        &distribution_setup.mint.pubkey(),
        DEFAULT_RECIPIENT_AMOUNT,
        &distribution_setup.token_program,
    );

    let setup1 = AddDirectRecipientSetup {
        authority: distribution_setup.authority.insecure_clone(),
        distribution_pda: distribution_setup.distribution_pda,
        recipient: recipient1,
        recipient_pda: recipient1_pda,
        recipient_bump: recipient1_bump,
        amount: DEFAULT_RECIPIENT_AMOUNT,
        schedule: schedule.clone(),
        token_program: distribution_setup.token_program,
        mint: distribution_setup.mint.pubkey(),
        distribution_vault: distribution_setup.distribution_vault,
        authority_token_account: authority_token_account1,
    };

    let instruction1 = setup1.build_instruction(&ctx);
    instruction1.send_expect_success(&mut ctx);

    assert_direct_recipient(
        &ctx,
        &setup1.recipient_pda,
        &setup1.recipient.pubkey(),
        DEFAULT_RECIPIENT_AMOUNT,
        0,
        setup1.recipient_bump,
    );

    let recipient2 = ctx.create_funded_keypair();
    let (recipient2_pda, recipient2_bump) =
        find_direct_recipient_pda(&distribution_setup.distribution_pda, &recipient2.pubkey());

    let authority_token_account2 = ctx.create_ata_for_program_with_balance(
        &distribution_setup.authority.pubkey(),
        &distribution_setup.mint.pubkey(),
        DEFAULT_RECIPIENT_AMOUNT * 2,
        &distribution_setup.token_program,
    );

    let setup2 = AddDirectRecipientSetup {
        authority: distribution_setup.authority,
        distribution_pda: distribution_setup.distribution_pda,
        recipient: recipient2,
        recipient_pda: recipient2_pda,
        recipient_bump: recipient2_bump,
        amount: DEFAULT_RECIPIENT_AMOUNT * 2,
        schedule: schedule.clone(),
        token_program: distribution_setup.token_program,
        mint: distribution_setup.mint.pubkey(),
        distribution_vault: distribution_setup.distribution_vault,
        authority_token_account: authority_token_account2,
    };

    let instruction2 = setup2.build_instruction(&ctx);
    instruction2.send_expect_success(&mut ctx);

    assert_direct_recipient(
        &ctx,
        &setup2.recipient_pda,
        &setup2.recipient.pubkey(),
        DEFAULT_RECIPIENT_AMOUNT * 2,
        0,
        setup2.recipient_bump,
    );
}

#[test]
fn test_add_direct_recipient_insufficient_funds() {
    let mut ctx = TestContext::new();
    let distribution_setup = CreateDirectDistributionSetup::new(&mut ctx);
    let create_instruction = distribution_setup.build_instruction(&ctx);
    create_instruction.send_expect_success(&mut ctx);

    let current_ts = ctx.get_current_timestamp();
    let recipient = ctx.create_funded_keypair();
    let (recipient_pda, recipient_bump) =
        find_direct_recipient_pda(&distribution_setup.distribution_pda, &recipient.pubkey());

    // Authority has only 500_000 tokens but tries to allocate 1_500_000
    let authority_token_account = ctx.create_ata_for_program_with_balance(
        &distribution_setup.authority.pubkey(),
        &distribution_setup.mint.pubkey(),
        500_000,
        &distribution_setup.token_program,
    );

    let setup = AddDirectRecipientSetup {
        authority: distribution_setup.authority,
        distribution_pda: distribution_setup.distribution_pda,
        recipient,
        recipient_pda,
        recipient_bump,
        amount: 1_500_000,
        schedule: VestingSchedule::Linear { start_ts: current_ts, end_ts: current_ts + 86400 * 365 },
        token_program: distribution_setup.token_program,
        mint: distribution_setup.mint.pubkey(),
        distribution_vault: distribution_setup.distribution_vault,
        authority_token_account,
    };

    let instruction = setup.build_instruction(&ctx);
    let _error = instruction.send_expect_error(&mut ctx);
}
