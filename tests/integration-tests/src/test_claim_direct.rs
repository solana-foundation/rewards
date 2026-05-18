use rewards_program_client::{
    accounts::{DirectDistribution, DirectRecipient},
    instructions::{
        AddDirectRecipientBuilder, ClaimDirectBuilder, CloseDirectRecipientBuilder, CreateDirectDistributionBuilder,
    },
    types::VestingSchedule,
};
use solana_sdk::{
    account::Account,
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_associated_token_account::{
    get_associated_token_address_with_program_id, instruction::create_associated_token_account_idempotent,
};
use spl_token_2022::{
    extension::{
        transfer_fee::{instruction::initialize_transfer_fee_config, TransferFeeAmount},
        BaseStateWithExtensions, ExtensionType, StateWithExtensions,
    },
    instruction::{initialize_mint2, mint_to_checked},
    state::{Account as Token2022Account, Mint as Token2022Mint},
    ID as TOKEN_2022_PROGRAM_ID,
};

use crate::fixtures::{ClaimDirectFixture, ClaimDirectSetup};
use crate::utils::{
    assert_account_closed, assert_direct_recipient, assert_instruction_error, assert_rewards_error,
    expected_linear_unlock, find_direct_distribution_pda, find_direct_recipient_pda, find_event_authority_pda,
    test_empty_data, test_missing_signer, test_not_writable, test_wrong_current_program, RewardsError, TestContext,
};

const TRANSFER_FEE_DECIMALS: u8 = 6;
const TRANSFER_FEE_BPS: u16 = 1_000;
const DIRECT_TRANSFER_FEE_GROSS_DEPOSIT: u64 = 1_000_000;

fn create_transfer_fee_mint(ctx: &mut TestContext, mint: &Keypair, decimals: u8, maximum_fee: u64) {
    let mint_len =
        ExtensionType::try_calculate_account_len::<Token2022Mint>(&[ExtensionType::TransferFeeConfig]).unwrap();

    ctx.svm
        .set_account(
            mint.pubkey(),
            Account {
                lamports: ctx.svm.minimum_balance_for_rent_exemption(mint_len),
                data: vec![0u8; mint_len],
                owner: TOKEN_2022_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            },
        )
        .unwrap();

    ctx.send_transaction(
        initialize_transfer_fee_config(
            &TOKEN_2022_PROGRAM_ID,
            &mint.pubkey(),
            Some(&ctx.payer.pubkey()),
            Some(&ctx.payer.pubkey()),
            TRANSFER_FEE_BPS,
            maximum_fee,
        )
        .unwrap(),
        &[],
    )
    .unwrap();

    ctx.send_transaction(
        initialize_mint2(&TOKEN_2022_PROGRAM_ID, &mint.pubkey(), &ctx.payer.pubkey(), None, decimals).unwrap(),
        &[],
    )
    .unwrap();
}

fn create_token_2022_ata(ctx: &mut TestContext, owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    let ata = get_associated_token_address_with_program_id(owner, mint, &TOKEN_2022_PROGRAM_ID);
    ctx.send_transaction(
        create_associated_token_account_idempotent(&ctx.payer.pubkey(), owner, mint, &TOKEN_2022_PROGRAM_ID),
        &[],
    )
    .unwrap();
    ata
}

fn mint_token_2022(ctx: &mut TestContext, mint: &Pubkey, destination: &Pubkey, amount: u64, decimals: u8) {
    ctx.send_transaction(
        mint_to_checked(&TOKEN_2022_PROGRAM_ID, mint, destination, &ctx.payer.pubkey(), &[], amount, decimals).unwrap(),
        &[],
    )
    .unwrap();
}

fn get_token_2022_amounts(ctx: &TestContext, account: &Pubkey) -> (u64, u64) {
    let account_data = ctx.get_account(account).expect("token account should exist");
    let parsed =
        StateWithExtensions::<Token2022Account>::unpack(&account_data.data).expect("token account should parse");
    let transfer_fee_amount = parsed.get_extension::<TransferFeeAmount>().expect("transfer fee extension should exist");
    (parsed.base.amount, u64::from(transfer_fee_amount.withheld_amount))
}

fn calculate_transfer_fee(amount: u64) -> u64 {
    amount * u64::from(TRANSFER_FEE_BPS) / 10_000
}

#[test]
fn test_claim_direct_missing_recipient_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<ClaimDirectFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_claim_direct_distribution_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimDirectFixture>(&mut ctx, 1);
}

#[test]
fn test_claim_direct_recipient_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimDirectFixture>(&mut ctx, 2);
}

#[test]
fn test_claim_direct_vault_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimDirectFixture>(&mut ctx, 4);
}

#[test]
fn test_claim_direct_recipient_token_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimDirectFixture>(&mut ctx, 5);
}

#[test]
fn test_claim_direct_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<ClaimDirectFixture>(&mut ctx);
}

#[test]
fn test_claim_direct_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<ClaimDirectFixture>(&mut ctx);
}

#[test]
fn test_claim_direct_success_full() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::new(&mut ctx);

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.recipient_token_account);
    assert_eq!(balance, setup.amount);

    assert_direct_recipient(
        &ctx,
        &setup.recipient_pda,
        &setup.recipient.pubkey(),
        setup.amount,
        setup.amount,
        setup.recipient_bump,
    );
}

#[test]
fn test_claim_direct_success_token_2022() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::new_token_2022(&mut ctx);

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.recipient_token_account);
    assert_eq!(balance, setup.amount);
}

#[test]
fn test_claim_direct_rejects_recipient_token_account_wrong_owner() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::new(&mut ctx);
    let attacker = ctx.create_funded_keypair();
    let attacker_token_account = ctx.create_ata_for_program(&attacker.pubkey(), &setup.mint, &setup.token_program);

    let test_ix = setup.build_instruction(&ctx).with_account_at(5, attacker_token_account);
    let error = test_ix.send_expect_error(&mut ctx);

    assert_instruction_error(error, InstructionError::InvalidAccountData);
    assert_eq!(ctx.get_token_balance(&attacker_token_account), 0);
    assert_eq!(ctx.get_token_balance(&setup.recipient_token_account), 0);
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
fn test_claim_direct_transfer_fee_mint_tracks_vault_debits() {
    let mut ctx = TestContext::new();
    let mint = Keypair::new();
    let distribution_authority = ctx.create_funded_keypair();
    let distribution_seed = Keypair::new();
    let recipient = ctx.create_funded_keypair();
    let original_payer = ctx.create_funded_keypair();
    let (event_authority, _) = find_event_authority_pda();

    create_transfer_fee_mint(&mut ctx, &mint, TRANSFER_FEE_DECIMALS, DIRECT_TRANSFER_FEE_GROSS_DEPOSIT);

    let (distribution_pda, distribution_bump) =
        find_direct_distribution_pda(&mint.pubkey(), &distribution_authority.pubkey(), &distribution_seed.pubkey());
    let distribution_vault =
        get_associated_token_address_with_program_id(&distribution_pda, &mint.pubkey(), &TOKEN_2022_PROGRAM_ID);
    let (recipient_pda, recipient_bump) = find_direct_recipient_pda(&distribution_pda, &recipient.pubkey());

    let mut create_distribution = CreateDirectDistributionBuilder::new();
    create_distribution
        .payer(ctx.payer.pubkey())
        .authority(distribution_authority.pubkey())
        .seeds(distribution_seed.pubkey())
        .distribution(distribution_pda)
        .mint(mint.pubkey())
        .distribution_vault(distribution_vault)
        .token_program(TOKEN_2022_PROGRAM_ID)
        .event_authority(event_authority)
        .bump(distribution_bump)
        .revocable(0)
        .clawback_ts(0);
    ctx.send_transaction(create_distribution.instruction(), &[&distribution_authority, &distribution_seed]).unwrap();

    let authority_token_account = create_token_2022_ata(&mut ctx, &distribution_authority.pubkey(), &mint.pubkey());
    mint_token_2022(
        &mut ctx,
        &mint.pubkey(),
        &authority_token_account,
        DIRECT_TRANSFER_FEE_GROSS_DEPOSIT,
        TRANSFER_FEE_DECIMALS,
    );

    let mut add_recipient = AddDirectRecipientBuilder::new();
    add_recipient
        .payer(original_payer.pubkey())
        .authority(distribution_authority.pubkey())
        .distribution(distribution_pda)
        .recipient_account(recipient_pda)
        .recipient(recipient.pubkey())
        .mint(mint.pubkey())
        .distribution_vault(distribution_vault)
        .authority_token_account(authority_token_account)
        .token_program(TOKEN_2022_PROGRAM_ID)
        .event_authority(event_authority)
        .bump(recipient_bump)
        .amount(DIRECT_TRANSFER_FEE_GROSS_DEPOSIT)
        .schedule(VestingSchedule::Immediate);
    ctx.send_transaction(add_recipient.instruction(), &[&original_payer, &distribution_authority]).unwrap();

    let vault_funded_amount =
        DIRECT_TRANSFER_FEE_GROSS_DEPOSIT - calculate_transfer_fee(DIRECT_TRANSFER_FEE_GROSS_DEPOSIT);
    let recipient_account = ctx.get_account(&recipient_pda).expect("recipient account should exist");
    let recipient_state = DirectRecipient::from_bytes(&recipient_account.data).expect("recipient account should parse");
    assert_eq!(recipient_state.total_amount, vault_funded_amount);
    assert_eq!(ctx.get_token_balance(&distribution_vault), vault_funded_amount);

    let recipient_token_account = create_token_2022_ata(&mut ctx, &recipient.pubkey(), &mint.pubkey());
    let mut claim = ClaimDirectBuilder::new();
    claim
        .recipient(recipient.pubkey())
        .distribution(distribution_pda)
        .recipient_account(recipient_pda)
        .mint(mint.pubkey())
        .distribution_vault(distribution_vault)
        .recipient_token_account(recipient_token_account)
        .token_program(TOKEN_2022_PROGRAM_ID)
        .event_authority(event_authority)
        .amount(0);
    ctx.send_transaction(claim.instruction(), &[&recipient]).unwrap();

    let claim_fee = calculate_transfer_fee(vault_funded_amount);
    let (recipient_spendable_balance, recipient_withheld_amount) =
        get_token_2022_amounts(&ctx, &recipient_token_account);
    assert_eq!(recipient_spendable_balance, vault_funded_amount - claim_fee);
    assert_eq!(recipient_withheld_amount, claim_fee);
    assert_eq!(ctx.get_token_balance(&distribution_vault), 0);

    let recipient_account = ctx.get_account(&recipient_pda).expect("recipient account should exist");
    let recipient_state = DirectRecipient::from_bytes(&recipient_account.data).expect("recipient account should parse");
    assert_eq!(recipient_state.total_amount, vault_funded_amount);
    assert_eq!(recipient_state.claimed_amount, vault_funded_amount);

    let distribution_account = ctx.get_account(&distribution_pda).expect("distribution account should exist");
    let distribution_state =
        DirectDistribution::from_bytes(&distribution_account.data).expect("distribution account should parse");
    assert_eq!(distribution_state.total_allocated, vault_funded_amount);
    assert_eq!(distribution_state.total_claimed, vault_funded_amount);

    let mut close_recipient = CloseDirectRecipientBuilder::new();
    close_recipient
        .recipient(recipient.pubkey())
        .original_payer(original_payer.pubkey())
        .distribution(distribution_pda)
        .recipient_account(recipient_pda)
        .event_authority(event_authority);
    ctx.send_transaction(close_recipient.instruction(), &[]).unwrap();

    assert_account_closed(&ctx, &recipient_pda);
}

#[test]
fn test_claim_direct_partial_25_percent() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::builder(&mut ctx).warp_to_end(false).build();

    let duration = setup.end_ts - setup.start_ts;
    let timestamp_25_percent = setup.start_ts + (duration / 4);
    ctx.warp_to_timestamp(timestamp_25_percent);

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.recipient_token_account);
    let expected = expected_linear_unlock(setup.amount, setup.start_ts, setup.end_ts, timestamp_25_percent);
    assert_eq!(balance, expected, "Balance should match exact linear unlock at 25%");
}

#[test]
fn test_claim_direct_partial_50_percent() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::builder(&mut ctx).warp_to_end(false).build();

    let duration = setup.end_ts - setup.start_ts;
    let timestamp_50_percent = setup.start_ts + (duration / 2);
    ctx.warp_to_timestamp(timestamp_50_percent);

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.recipient_token_account);
    let expected = expected_linear_unlock(setup.amount, setup.start_ts, setup.end_ts, timestamp_50_percent);
    assert_eq!(balance, expected, "Balance should match exact linear unlock at 50%");
}

#[test]
fn test_claim_direct_multiple_claims() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::builder(&mut ctx).warp_to_end(false).build();

    let duration = setup.end_ts - setup.start_ts;

    let timestamp_25_percent = setup.start_ts + (duration / 4);
    ctx.warp_to_timestamp(timestamp_25_percent);

    let test_ix1 = setup.build_instruction(&ctx);
    test_ix1.send_expect_success(&mut ctx);

    let balance_after_first = ctx.get_token_balance(&setup.recipient_token_account);
    let expected_first = expected_linear_unlock(setup.amount, setup.start_ts, setup.end_ts, timestamp_25_percent);
    assert_eq!(balance_after_first, expected_first, "Balance should match exact linear unlock at 25%");

    let timestamp_50_percent = setup.start_ts + (duration / 2);
    ctx.warp_to_timestamp(timestamp_50_percent);

    let test_ix2 = setup.build_instruction(&ctx);
    test_ix2.send_expect_success(&mut ctx);

    let balance_after_second = ctx.get_token_balance(&setup.recipient_token_account);
    let expected_total = expected_linear_unlock(setup.amount, setup.start_ts, setup.end_ts, timestamp_50_percent);
    assert_eq!(balance_after_second, expected_total, "Balance should match exact linear unlock at 50%");
}

#[test]
fn test_claim_direct_nothing_before_start() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();

    let setup = ClaimDirectSetup::builder(&mut ctx)
        .schedule(VestingSchedule::Linear { start_ts: current_ts + 1000, end_ts: current_ts + 2000 })
        .warp_to_end(false)
        .build();

    let test_ix = setup.build_instruction(&ctx);
    let error = test_ix.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::NothingToClaim);
}

#[test]
fn test_claim_direct_nothing_already_claimed() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::new(&mut ctx);

    let test_ix1 = setup.build_instruction(&ctx);
    test_ix1.send_expect_success(&mut ctx);

    ctx.advance_slot();

    let test_ix2 = setup.build_instruction(&ctx);
    let error = test_ix2.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::NothingToClaim);
}

#[test]
fn test_claim_direct_unauthorized() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::new(&mut ctx);

    let wrong_signer = ctx.create_funded_keypair();
    let wrong_signer_token_account = ctx.create_token_account(&wrong_signer.pubkey(), &setup.mint);

    let test_ix = setup.build_instruction_with_wrong_signer(&ctx, &wrong_signer, wrong_signer_token_account);
    let error = test_ix.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::UnauthorizedRecipient);
}

#[test]
fn test_claim_direct_immediate_vesting() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::builder(&mut ctx).schedule(VestingSchedule::Immediate).warp_to_end(false).build();

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.recipient_token_account);
    assert_eq!(balance, setup.amount);

    assert_direct_recipient(
        &ctx,
        &setup.recipient_pda,
        &setup.recipient.pubkey(),
        setup.amount,
        setup.amount,
        setup.recipient_bump,
    );
}

#[test]
fn test_claim_direct_specific_amount() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::new(&mut ctx);

    let claim_amount = setup.amount / 3;
    let test_ix = setup.build_instruction_with_amount(claim_amount);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.recipient_token_account);
    assert_eq!(balance, claim_amount);

    assert_direct_recipient(
        &ctx,
        &setup.recipient_pda,
        &setup.recipient.pubkey(),
        setup.amount,
        claim_amount,
        setup.recipient_bump,
    );
}

#[test]
fn test_claim_direct_exceeds_claimable_amount() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::builder(&mut ctx).warp_to_end(false).build();

    // Warp to 50%
    let duration = setup.end_ts - setup.start_ts;
    let mid_point = setup.start_ts + (duration / 2);
    ctx.warp_to_timestamp(mid_point);

    // Request more than the 50% that's vested
    let test_ix = setup.build_instruction_with_amount(setup.amount);
    let error = test_ix.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::ExceedsClaimableAmount);
}

#[test]
fn test_claim_direct_rejects_mint_mismatch() {
    let mut ctx = TestContext::new();
    let setup = ClaimDirectSetup::new(&mut ctx);

    let wrong_mint = Keypair::new();
    ctx.create_mint_for_program(&wrong_mint, &ctx.payer.pubkey(), 6, &setup.token_program);

    let wrong_distribution_vault = ctx.create_ata_for_program_with_balance(
        &setup.distribution_pda,
        &wrong_mint.pubkey(),
        setup.amount,
        &setup.token_program,
    );
    let wrong_recipient_token_account =
        ctx.create_ata_for_program(&setup.recipient.pubkey(), &wrong_mint.pubkey(), &setup.token_program);

    let test_ix = setup
        .build_instruction(&ctx)
        .with_account_at(3, wrong_mint.pubkey())
        .with_account_at(4, wrong_distribution_vault)
        .with_account_at(5, wrong_recipient_token_account);

    let error = test_ix.send_expect_error(&mut ctx);
    assert_instruction_error(error, InstructionError::InvalidAccountData);
}
