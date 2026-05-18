use solana_sdk::signature::Signer;

use crate::fixtures::{CloseMerkleDistributionFixture, CloseMerkleDistributionSetup};
use crate::utils::{
    assert_account_closed, assert_merkle_distribution_closed, assert_rewards_error, test_empty_data,
    test_missing_signer, test_not_writable, test_wrong_current_program, RewardsError, TestContext,
};

#[test]
fn test_close_merkle_distribution_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<CloseMerkleDistributionFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_close_merkle_distribution_distribution_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CloseMerkleDistributionFixture>(&mut ctx, 1);
}

#[test]
fn test_close_merkle_distribution_distribution_vault_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CloseMerkleDistributionFixture>(&mut ctx, 3);
}

#[test]
fn test_close_merkle_distribution_authority_token_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CloseMerkleDistributionFixture>(&mut ctx, 4);
}

#[test]
fn test_close_merkle_distribution_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<CloseMerkleDistributionFixture>(&mut ctx);
}

#[test]
fn test_close_merkle_distribution_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<CloseMerkleDistributionFixture>(&mut ctx);
}

#[test]
fn test_close_merkle_distribution_success() {
    let mut ctx = TestContext::new();
    let setup = CloseMerkleDistributionSetup::new(&mut ctx);

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    assert_merkle_distribution_closed(&ctx, &setup.distribution_pda);
}

#[test]
fn test_close_merkle_distribution_success_token_2022() {
    let mut ctx = TestContext::new();
    let setup = CloseMerkleDistributionSetup::new_token_2022(&mut ctx);

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    assert_merkle_distribution_closed(&ctx, &setup.distribution_pda);
}

#[test]
fn test_close_merkle_distribution_unauthorized() {
    let mut ctx = TestContext::new();
    let setup = CloseMerkleDistributionSetup::new(&mut ctx);

    let wrong_authority = ctx.create_funded_keypair();
    let wrong_token_account = ctx.create_token_account(&wrong_authority.pubkey(), &setup.mint);

    let test_ix = setup.build_instruction_with_wrong_authority(&ctx, &wrong_authority, wrong_token_account);
    let error = test_ix.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}

#[test]
fn test_close_merkle_distribution_before_clawback() {
    let mut ctx = TestContext::new();
    let setup = CloseMerkleDistributionSetup::builder(&mut ctx).warp_to_clawback(false).build();

    let test_ix = setup.build_instruction(&ctx);
    let error = test_ix.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::ClawbackNotReached);
}

#[test]
fn test_close_merkle_distribution_returns_tokens() {
    let mut ctx = TestContext::new();
    let fund_amount = 1_000_000u64;

    let setup = CloseMerkleDistributionSetup::builder(&mut ctx).amount(fund_amount).build();

    let distribution_vault_balance = ctx.get_token_balance(&setup.distribution_vault);
    assert_eq!(distribution_vault_balance, fund_amount, "Vault should have funded tokens");

    let authority_balance_before = ctx.get_token_balance(&setup.authority_token_account);

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let authority_balance_after = ctx.get_token_balance(&setup.authority_token_account);
    assert_eq!(
        authority_balance_after,
        authority_balance_before + fund_amount,
        "Authority should receive vault tokens"
    );
}

#[test]
fn test_close_merkle_distribution_returns_tokens_token_2022() {
    let mut ctx = TestContext::new();
    let fund_amount = 1_000_000u64;

    let setup = CloseMerkleDistributionSetup::builder(&mut ctx).token_2022().amount(fund_amount).build();

    let distribution_vault_balance = ctx.get_token_balance(&setup.distribution_vault);
    assert_eq!(distribution_vault_balance, fund_amount, "Vault should have funded tokens");

    let authority_balance_before = ctx.get_token_balance(&setup.authority_token_account);

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let authority_balance_after = ctx.get_token_balance(&setup.authority_token_account);
    assert_eq!(
        authority_balance_after,
        authority_balance_before + fund_amount,
        "Authority should receive vault tokens"
    );
}

#[test]
fn test_close_merkle_distribution_closes_distribution_vault() {
    let mut ctx = TestContext::new();
    let setup = CloseMerkleDistributionSetup::new(&mut ctx);

    // Verify distribution vault exists before close
    assert!(ctx.get_account(&setup.distribution_vault).is_some(), "Distribution vault should exist before close");

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    // Vault should be closed
    assert_account_closed(&ctx, &setup.distribution_vault);
}
