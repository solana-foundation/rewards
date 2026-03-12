use crate::fixtures::{
    build_set_continuous_merkle_root_instruction, SetContinuousMerkleRootFixture, SetContinuousMerkleRootSetup,
};
use crate::utils::{
    assert_rewards_error, get_reward_pool, test_empty_data, test_missing_signer, test_not_writable,
    test_truncated_data, test_wrong_current_program, RewardsError, TestContext, TestInstruction,
};
use solana_sdk::signature::Signer;

#[test]
fn test_set_continuous_merkle_root_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<SetContinuousMerkleRootFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_set_continuous_merkle_root_pool_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<SetContinuousMerkleRootFixture>(&mut ctx, 1);
}

#[test]
fn test_set_continuous_merkle_root_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<SetContinuousMerkleRootFixture>(&mut ctx);
}

#[test]
fn test_set_continuous_merkle_root_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<SetContinuousMerkleRootFixture>(&mut ctx);
}

#[test]
fn test_set_continuous_merkle_root_truncated_data() {
    let mut ctx = TestContext::new();
    test_truncated_data::<SetContinuousMerkleRootFixture>(&mut ctx);
}

#[test]
fn test_set_continuous_merkle_root_success() {
    let mut ctx = TestContext::new();
    let setup = SetContinuousMerkleRootSetup::new(&mut ctx);

    setup.build_instruction().send_expect_success(&mut ctx);

    let pool = get_reward_pool(&ctx, &setup.pool_setup.reward_pool_pda);
    assert_eq!(pool.merkle_root, setup.merkle_root);
    assert_eq!(pool.merkle_root_epoch, setup.epoch);
}

#[test]
fn test_set_continuous_merkle_root_requires_monotonic_epoch() {
    let mut ctx = TestContext::new();
    let setup = SetContinuousMerkleRootSetup::new(&mut ctx);

    setup.build_instruction().send_expect_success(&mut ctx);

    let ix = build_set_continuous_merkle_root_instruction(&setup.pool_setup, [22u8; 32], setup.epoch);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::InvalidMerkleRootEpoch);
}

#[test]
fn test_set_continuous_merkle_root_rotation_success() {
    let mut ctx = TestContext::new();
    let setup = SetContinuousMerkleRootSetup::new(&mut ctx);

    setup.build_instruction().send_expect_success(&mut ctx);

    let new_root = [33u8; 32];
    let new_epoch = setup.epoch + 1;
    build_set_continuous_merkle_root_instruction(&setup.pool_setup, new_root, new_epoch).send_expect_success(&mut ctx);

    let pool = get_reward_pool(&ctx, &setup.pool_setup.reward_pool_pda);
    assert_eq!(pool.merkle_root, new_root);
    assert_eq!(pool.merkle_root_epoch, new_epoch);
}

#[test]
fn test_set_continuous_merkle_root_unauthorized_authority() {
    let mut ctx = TestContext::new();
    let setup = SetContinuousMerkleRootSetup::new(&mut ctx);
    let wrong_authority = ctx.create_funded_keypair();

    let mut builder = rewards_program_client::instructions::SetContinuousMerkleRootBuilder::new();
    builder
        .authority(wrong_authority.pubkey())
        .reward_pool(setup.pool_setup.reward_pool_pda)
        .merkle_root(setup.merkle_root)
        .epoch(setup.epoch);

    let ix = TestInstruction {
        instruction: builder.instruction(),
        signers: vec![wrong_authority],
        name: "SetContinuousMerkleRoot",
    };
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}

#[test]
fn test_set_continuous_merkle_root_epoch_zero_invalid() {
    let mut ctx = TestContext::new();
    let setup = SetContinuousMerkleRootSetup::new(&mut ctx);

    let ix = build_set_continuous_merkle_root_instruction(&setup.pool_setup, [44u8; 32], 0);
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::InvalidMerkleRootEpoch);
}
