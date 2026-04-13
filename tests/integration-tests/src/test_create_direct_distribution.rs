use solana_sdk::{account::Account, signer::Signer};
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;

use crate::fixtures::{CloseDirectDistributionSetup, CreateDirectDistributionFixture, CreateDirectDistributionSetup};
use crate::utils::{
    assert_direct_distribution, assert_rewards_error, test_empty_data, test_missing_signer, test_not_writable,
    test_truncated_data, test_wrong_current_program, test_wrong_system_program, RewardsError, TestContext,
};

#[test]
fn test_create_direct_distribution_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<CreateDirectDistributionFixture>(&mut ctx, 1, 0);
}

#[test]
fn test_create_direct_distribution_missing_seeds_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<CreateDirectDistributionFixture>(&mut ctx, 2, 1);
}

#[test]
fn test_create_direct_distribution_distribution_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CreateDirectDistributionFixture>(&mut ctx, 3);
}

#[test]
fn test_create_direct_distribution_distribution_vault_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CreateDirectDistributionFixture>(&mut ctx, 6);
}

#[test]
fn test_create_direct_distribution_wrong_system_program() {
    let mut ctx = TestContext::new();
    test_wrong_system_program::<CreateDirectDistributionFixture>(&mut ctx);
}

#[test]
fn test_create_direct_distribution_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<CreateDirectDistributionFixture>(&mut ctx);
}

#[test]
fn test_create_direct_distribution_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<CreateDirectDistributionFixture>(&mut ctx);
}

#[test]
fn test_create_direct_distribution_truncated_data() {
    let mut ctx = TestContext::new();
    test_truncated_data::<CreateDirectDistributionFixture>(&mut ctx);
}

#[test]
fn test_create_direct_distribution_success() {
    let mut ctx = TestContext::new();
    let setup = CreateDirectDistributionSetup::new(&mut ctx);
    let instruction = setup.build_instruction(&ctx);

    instruction.send_expect_success(&mut ctx);

    assert_direct_distribution(
        &ctx,
        &setup.distribution_pda,
        &setup.authority.pubkey(),
        &setup.mint.pubkey(),
        setup.bump,
    );
}

#[test]
fn test_create_direct_distribution_success_token_2022() {
    let mut ctx = TestContext::new();
    let setup = CreateDirectDistributionSetup::new_token_2022(&mut ctx);
    let instruction = setup.build_instruction(&ctx);

    instruction.send_expect_success(&mut ctx);

    assert_direct_distribution(
        &ctx,
        &setup.distribution_pda,
        &setup.authority.pubkey(),
        &setup.mint.pubkey(),
        setup.bump,
    );
}

#[test]
fn test_create_direct_distribution_prefunded_distribution_pda() {
    let mut ctx = TestContext::new();
    let setup = CreateDirectDistributionSetup::new(&mut ctx);

    ctx.svm
        .set_account(
            setup.distribution_pda,
            Account { lamports: 1, data: vec![], owner: SYSTEM_PROGRAM_ID, executable: false, rent_epoch: 0 },
        )
        .unwrap();

    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    assert_direct_distribution(
        &ctx,
        &setup.distribution_pda,
        &setup.authority.pubkey(),
        &setup.mint.pubkey(),
        setup.bump,
    );
}

#[test]
fn test_create_direct_distribution_fails_if_closed_before() {
    let mut ctx = TestContext::new();
    let setup = CreateDirectDistributionSetup::new(&mut ctx);
    let create_ix = setup.build_instruction(&ctx);
    create_ix.send_expect_success(&mut ctx);

    let authority_token_account =
        ctx.create_ata_for_program(&setup.authority.pubkey(), &setup.mint.pubkey(), &setup.token_program);
    let close_setup = CloseDirectDistributionSetup {
        authority: setup.authority.insecure_clone(),
        distribution_pda: setup.distribution_pda,
        tombstone_pda: setup.tombstone_pda,
        mint: setup.mint.pubkey(),
        distribution_vault: setup.distribution_vault,
        authority_token_account,
        token_program: setup.token_program,
    };
    close_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    ctx.advance_slot();
    let recreate_ix = setup.build_instruction(&ctx);
    let error = recreate_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::DistributionPermanentlyClosed);
}
