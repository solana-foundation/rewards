use solana_sdk::{account::Account, signer::Signer};
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;

use crate::fixtures::{CreateDirectDistributionFixture, CreateDirectDistributionSetup};
use crate::utils::{
    assert_direct_distribution, test_empty_data, test_missing_signer, test_not_writable, test_truncated_data,
    test_wrong_current_program, test_wrong_system_program, TestContext,
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
    test_not_writable::<CreateDirectDistributionFixture>(&mut ctx, 5);
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
