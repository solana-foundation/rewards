use solana_sdk::{account::Account, signer::Signer};
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;

use crate::fixtures::{CloseMerkleDistributionSetup, CreateMerkleDistributionFixture, CreateMerkleDistributionSetup};
use crate::utils::{
    assert_merkle_distribution, assert_rewards_error, test_empty_data, test_missing_signer, test_not_writable,
    test_truncated_data, test_wrong_current_program, test_wrong_system_program, RewardsError, TestContext,
};

#[test]
fn test_create_merkle_distribution_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<CreateMerkleDistributionFixture>(&mut ctx, 1, 0);
}

#[test]
fn test_create_merkle_distribution_missing_seeds_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<CreateMerkleDistributionFixture>(&mut ctx, 2, 1);
}

#[test]
fn test_create_merkle_distribution_distribution_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CreateMerkleDistributionFixture>(&mut ctx, 3);
}

#[test]
fn test_create_merkle_distribution_distribution_vault_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CreateMerkleDistributionFixture>(&mut ctx, 5);
}

#[test]
fn test_create_merkle_distribution_authority_token_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CreateMerkleDistributionFixture>(&mut ctx, 6);
}

#[test]
fn test_create_merkle_distribution_wrong_system_program() {
    let mut ctx = TestContext::new();
    test_wrong_system_program::<CreateMerkleDistributionFixture>(&mut ctx);
}

#[test]
fn test_create_merkle_distribution_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<CreateMerkleDistributionFixture>(&mut ctx);
}

#[test]
fn test_create_merkle_distribution_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<CreateMerkleDistributionFixture>(&mut ctx);
}

#[test]
fn test_create_merkle_distribution_truncated_data() {
    let mut ctx = TestContext::new();
    test_truncated_data::<CreateMerkleDistributionFixture>(&mut ctx);
}

#[test]
fn test_create_merkle_distribution_success() {
    let mut ctx = TestContext::new();
    let setup = CreateMerkleDistributionSetup::new(&mut ctx);
    let instruction = setup.build_instruction(&ctx);

    instruction.send_expect_success(&mut ctx);

    assert_merkle_distribution(
        &ctx,
        &setup.distribution_pda,
        &setup.authority.pubkey(),
        &setup.mint.pubkey(),
        &setup.merkle_root,
        setup.total_amount,
        setup.bump,
    );
}

#[test]
fn test_create_merkle_distribution_fails_if_closed_before() {
    let mut ctx = TestContext::new();
    let setup = CreateMerkleDistributionSetup::new(&mut ctx);
    let create_ix = setup.build_instruction(&ctx);
    create_ix.send_expect_success(&mut ctx);

    ctx.warp_to_timestamp(setup.clawback_ts);
    let authority_token_account =
        ctx.create_ata_for_program(&setup.authority.pubkey(), &setup.mint.pubkey(), &setup.token_program);
    let close_setup = CloseMerkleDistributionSetup {
        authority: setup.authority.insecure_clone(),
        distribution_pda: setup.distribution_pda,
        mint: setup.mint.pubkey(),
        distribution_vault: setup.distribution_vault,
        authority_token_account,
        token_program: setup.token_program,
        funded_amount: setup.amount,
        clawback_ts: setup.clawback_ts,
    };
    close_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    ctx.advance_slot();
    let recreate_ix = setup.build_instruction(&ctx);
    let error = recreate_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::DistributionPermanentlyClosed);
}

#[test]
fn test_create_merkle_distribution_success_token_2022() {
    let mut ctx = TestContext::new();
    let setup = CreateMerkleDistributionSetup::new_token_2022(&mut ctx);
    let instruction = setup.build_instruction(&ctx);

    instruction.send_expect_success(&mut ctx);

    assert_merkle_distribution(
        &ctx,
        &setup.distribution_pda,
        &setup.authority.pubkey(),
        &setup.mint.pubkey(),
        &setup.merkle_root,
        setup.total_amount,
        setup.bump,
    );
}

#[test]
fn test_create_merkle_distribution_zero_amount() {
    let mut ctx = TestContext::new();
    let setup = CreateMerkleDistributionSetup::builder(&mut ctx).amount(0).build();

    let instruction = setup.build_instruction(&ctx);
    let error = instruction.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::InvalidAmount);
}

#[test]
fn test_create_merkle_distribution_zero_total_amount() {
    let mut ctx = TestContext::new();
    let setup = CreateMerkleDistributionSetup::builder(&mut ctx).total_amount(0).build();

    let instruction = setup.build_instruction(&ctx);
    let error = instruction.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::InvalidAmount);
}

#[test]
fn test_create_merkle_distribution_funds_distribution_vault() {
    let mut ctx = TestContext::new();
    let setup = CreateMerkleDistributionSetup::new(&mut ctx);
    let amount = setup.amount;

    let distribution_vault_balance_before = ctx.get_token_balance(&setup.distribution_vault);
    assert_eq!(distribution_vault_balance_before, 0);

    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    let distribution_vault_balance_after = ctx.get_token_balance(&setup.distribution_vault);
    assert_eq!(distribution_vault_balance_after, amount);
}

#[test]
fn test_create_merkle_distribution_custom_merkle_root() {
    let mut ctx = TestContext::new();
    let custom_root = [42u8; 32];
    let setup = CreateMerkleDistributionSetup::builder(&mut ctx).merkle_root(custom_root).build();

    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    assert_merkle_distribution(
        &ctx,
        &setup.distribution_pda,
        &setup.authority.pubkey(),
        &setup.mint.pubkey(),
        &custom_root,
        setup.total_amount,
        setup.bump,
    );
}

#[test]
fn test_create_merkle_distribution_custom_clawback() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();
    let custom_clawback = current_ts + 86400 * 30; // 30 days

    let setup = CreateMerkleDistributionSetup::builder(&mut ctx).clawback_ts(custom_clawback).build();

    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    assert_merkle_distribution(
        &ctx,
        &setup.distribution_pda,
        &setup.authority.pubkey(),
        &setup.mint.pubkey(),
        &setup.merkle_root,
        setup.total_amount,
        setup.bump,
    );
}

#[test]
fn test_create_merkle_distribution_prefunded_distribution_pda() {
    let mut ctx = TestContext::new();
    let setup = CreateMerkleDistributionSetup::new(&mut ctx);

    ctx.svm
        .set_account(
            setup.distribution_pda,
            Account { lamports: 1, data: vec![], owner: SYSTEM_PROGRAM_ID, executable: false, rent_epoch: 0 },
        )
        .unwrap();

    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    assert_merkle_distribution(
        &ctx,
        &setup.distribution_pda,
        &setup.authority.pubkey(),
        &setup.mint.pubkey(),
        &setup.merkle_root,
        setup.total_amount,
        setup.bump,
    );
}
