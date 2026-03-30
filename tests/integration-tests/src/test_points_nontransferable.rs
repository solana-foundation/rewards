use solana_sdk::signature::Signer;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;

use crate::fixtures::{InitPointsSetup, IssuePointsSetup};
use crate::utils::{TestContext, TestInstruction};

/// Test that a standard SPL TransferChecked on the points mint fails
/// because the mint has the NonTransferable extension. The user should
/// NOT be able to bypass the program's transfer gate by calling Token-2022
/// directly.
#[test]
fn test_points_direct_spl_transfer_blocked() {
    let mut ctx = TestContext::new();

    // Create a non-transferable points config (transferable=0)
    let init_setup = InitPointsSetup::builder(&mut ctx).transferable(0).build();
    init_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let points_mint = init_setup.points_mint_pda;

    // Issue 1000 to user
    let user = ctx.create_funded_keypair();
    let user_ata = get_associated_token_address_with_program_id(&user.pubkey(), &points_mint, &TOKEN_2022_PROGRAM_ID);

    let issue = IssuePointsSetup {
        authority: init_setup.authority.insecure_clone(),
        points_config_pda: init_setup.points_config_pda,
        points_mint_pda: points_mint,
        user: user.pubkey(),
        user_ata,
        quantity: 1_000,
    };
    issue.build_instruction(&ctx).send_expect_success(&mut ctx);

    // Issue to a second user to ensure they have an ATA
    let user2 = ctx.create_funded_keypair();
    let user2_ata = get_associated_token_address_with_program_id(&user2.pubkey(), &points_mint, &TOKEN_2022_PROGRAM_ID);

    let issue2 = IssuePointsSetup {
        authority: init_setup.authority.insecure_clone(),
        points_config_pda: init_setup.points_config_pda,
        points_mint_pda: points_mint,
        user: user2.pubkey(),
        user_ata: user2_ata,
        quantity: 100,
    };
    issue2.build_instruction(&ctx).send_expect_success(&mut ctx);

    // Attempt a direct Token-2022 TransferChecked from user to user2
    // This should fail because the mint has NonTransferable extension
    let transfer_ix = spl_token_2022::instruction::transfer_checked(
        &TOKEN_2022_PROGRAM_ID,
        &user_ata,
        &points_mint,
        &user2_ata,
        &user.pubkey(),
        &[],
        500,
        0, // decimals
    )
    .unwrap();

    let ix =
        TestInstruction { instruction: transfer_ix, signers: vec![user.insecure_clone()], name: "DirectSPLTransfer" };

    // This should fail - NonTransferable mint rejects direct transfers
    let _error = ix.send_expect_error(&mut ctx);
}

/// Verify that the points mint account exists and is owned by Token-2022
#[test]
fn test_points_mint_owned_by_token_2022() {
    let mut ctx = TestContext::new();

    let init_setup = InitPointsSetup::builder(&mut ctx).build();
    init_setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let mint_account = ctx.get_account(&init_setup.points_mint_pda).expect("Mint should exist");
    assert_eq!(mint_account.owner, TOKEN_2022_PROGRAM_ID, "Mint should be owned by Token-2022");
}
