use rewards_program_client::instructions::RevokePointsBuilder;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;

use crate::fixtures::IssuePointsSetup;
use crate::utils::{find_event_authority_pda, InstructionTestFixture, TestContext, TestInstruction};

pub const DEFAULT_REVOKE_ISSUE_QUANTITY: u64 = 500;

pub struct RevokePointsSetup {
    pub authority: Keypair,
    pub user: Pubkey,
    pub points_config_pda: Pubkey,
    pub points_mint_pda: Pubkey,
    pub user_ata: Pubkey,
    pub issued_quantity: u64,
}

impl RevokePointsSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        Self::new_with_quantity(ctx, DEFAULT_REVOKE_ISSUE_QUANTITY)
    }

    pub fn new_with_quantity(ctx: &mut TestContext, issue_quantity: u64) -> Self {
        // Create config with revocable=1
        let init_setup = crate::fixtures::InitPointsSetup::builder(ctx).revocable(1).build();
        let init_ix = init_setup.build_instruction(ctx);
        init_ix.send_expect_success(ctx);

        let user = Keypair::new();
        let user_ata = get_associated_token_address_with_program_id(
            &user.pubkey(),
            &init_setup.points_mint_pda,
            &TOKEN_2022_PROGRAM_ID,
        );

        // Issue points to user
        let issue_setup = IssuePointsSetup {
            authority: init_setup.authority.insecure_clone(),
            points_config_pda: init_setup.points_config_pda,
            points_mint_pda: init_setup.points_mint_pda,
            user: user.pubkey(),
            user_ata,
            quantity: issue_quantity,
        };
        let issue_ix = issue_setup.build_instruction(ctx);
        issue_ix.send_expect_success(ctx);

        RevokePointsSetup {
            authority: init_setup.authority,
            user: user.pubkey(),
            points_config_pda: init_setup.points_config_pda,
            points_mint_pda: init_setup.points_mint_pda,
            user_ata,
            issued_quantity: issue_quantity,
        }
    }

    pub fn build_instruction(&self, _ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = RevokePointsBuilder::new();
        builder
            .authority(self.authority.pubkey())
            .points_config(self.points_config_pda)
            .points_mint(self.points_mint_pda)
            .user(self.user)
            .user_token_account(self.user_ata)
            .token2022_program(TOKEN_2022_PROGRAM_ID)
            .event_authority(event_authority);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone()],
            name: "RevokePoints",
        }
    }
}

pub struct RevokePointsFixture;

impl InstructionTestFixture for RevokePointsFixture {
    const INSTRUCTION_NAME: &'static str = "RevokePoints";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = RevokePointsSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    /// 0: authority
    fn required_signers() -> &'static [usize] {
        &[0]
    }

    /// 2: points_mint, 4: user_token_account
    fn required_writable() -> &'static [usize] {
        &[2, 4]
    }

    fn system_program_index() -> Option<usize> {
        None
    }

    fn current_program_index() -> Option<usize> {
        Some(7)
    }

    fn data_len() -> usize {
        1 // discriminator only
    }
}
