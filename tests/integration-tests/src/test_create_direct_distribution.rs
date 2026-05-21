use rewards_program_client::instructions::CreateDirectDistributionBuilder;
use solana_sdk::{account::Account, instruction::InstructionError, pubkey::Pubkey, signature::Keypair, signer::Signer};
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

use crate::fixtures::{CloseDirectDistributionSetup, CreateDirectDistributionFixture, CreateDirectDistributionSetup};
use crate::utils::{
    assert_direct_distribution, assert_instruction_error, assert_rewards_error, find_direct_distribution_pda,
    find_event_authority_pda, test_empty_data, test_missing_signer, test_not_writable, test_truncated_data,
    test_wrong_current_program, test_wrong_system_program, RewardsError, TestContext, TestInstruction,
    TOKEN_2022_PROGRAM_ID,
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
fn test_create_direct_distribution_payer_can_be_seed() {
    let mut ctx = TestContext::new();
    let authority = ctx.create_funded_keypair();
    let mint = Keypair::new();

    ctx.create_mint_for_program(&mint, &ctx.payer.pubkey(), 6, &TOKEN_PROGRAM_ID);

    let (distribution_pda, bump) =
        find_direct_distribution_pda(&mint.pubkey(), &authority.pubkey(), &ctx.payer.pubkey());
    let distribution_vault = ctx.create_ata_for_program(&distribution_pda, &mint.pubkey(), &TOKEN_PROGRAM_ID);
    let (event_authority, _) = find_event_authority_pda();

    let mut builder = CreateDirectDistributionBuilder::new();
    builder
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .seeds(ctx.payer.pubkey())
        .distribution(distribution_pda)
        .mint(mint.pubkey())
        .distribution_vault(distribution_vault)
        .token_program(TOKEN_PROGRAM_ID)
        .event_authority(event_authority)
        .bump(bump)
        .revocable(0)
        .clawback_ts(0);

    let instruction = TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone()],
        name: "CreateDirectDistribution",
    };

    instruction.send_expect_success(&mut ctx);

    assert_direct_distribution(&ctx, &distribution_pda, &authority.pubkey(), &mint.pubkey(), bump);
}

#[test]
fn test_create_direct_distribution_rejects_unrelated_writable_seed() {
    let mut ctx = TestContext::new();
    let setup = CreateDirectDistributionSetup::new(&mut ctx);
    let mut instruction = setup.build_instruction(&ctx);

    instruction.instruction.accounts[2].is_writable = true;
    let error = instruction.send_expect_error(&mut ctx);

    assert_instruction_error(error, InstructionError::AccountBorrowFailed);
}

#[test]
fn test_create_direct_distribution_rejects_transfer_hook_mint() {
    let mut ctx = TestContext::new();
    let authority = ctx.create_funded_keypair();
    let seed = Keypair::new();
    let mint = Keypair::new();
    let hook_program_id = Pubkey::new_unique();

    ctx.create_token_2022_transfer_hook_mint(&mint, &ctx.payer.pubkey(), 6, &hook_program_id);

    let (distribution_pda, bump) = find_direct_distribution_pda(&mint.pubkey(), &authority.pubkey(), &seed.pubkey());
    let distribution_vault =
        get_associated_token_address_with_program_id(&distribution_pda, &mint.pubkey(), &TOKEN_2022_PROGRAM_ID);
    let (event_authority, _) = find_event_authority_pda();

    let mut builder = CreateDirectDistributionBuilder::new();
    builder
        .payer(ctx.payer.pubkey())
        .authority(authority.pubkey())
        .seeds(seed.pubkey())
        .distribution(distribution_pda)
        .mint(mint.pubkey())
        .distribution_vault(distribution_vault)
        .token_program(TOKEN_2022_PROGRAM_ID)
        .event_authority(event_authority)
        .bump(bump)
        .revocable(0)
        .clawback_ts(0);

    let create_ix = TestInstruction {
        instruction: builder.instruction(),
        signers: vec![authority.insecure_clone(), seed.insecure_clone()],
        name: "CreateDirectDistribution",
    };
    let error = create_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::TransferHookMintUnsupported);
    assert!(ctx.get_account(&distribution_pda).is_none(), "failed create should roll back distribution PDA");
}

#[test]
fn test_create_direct_distribution_prefunded_distribution_pda() {
    let mut ctx = TestContext::new();
    let setup = CreateDirectDistributionSetup::new(&mut ctx);

    ctx.set_account(
        setup.distribution_pda,
        Account { lamports: 1, data: vec![], owner: SYSTEM_PROGRAM_ID, executable: false, rent_epoch: 0 },
    );

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
