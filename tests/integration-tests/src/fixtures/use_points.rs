use rewards_program_client::instructions::UsePointsBuilder;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;

use crate::fixtures::{IssuePointsSetup, DEFAULT_ISSUE_QUANTITY};
use crate::utils::{find_event_authority_pda, InstructionTestFixture, TestContext, TestInstruction};

pub struct UsePointsSetup {
    pub authority: Keypair,
    pub user: Keypair,
    pub points_config_pda: Pubkey,
    pub points_mint_pda: Pubkey,
    pub user_ata: Pubkey,
    pub quantity: u64,
    pub issued_quantity: u64,
}

impl UsePointsSetup {
    pub fn builder(ctx: &mut TestContext) -> UsePointsSetupBuilder<'_> {
        UsePointsSetupBuilder::new(ctx)
    }

    pub fn new(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).build()
    }

    pub fn build_instruction(&self, _ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = UsePointsBuilder::new();
        builder
            .authority(self.authority.pubkey())
            .user(self.user.pubkey())
            .points_config(self.points_config_pda)
            .points_mint(self.points_mint_pda)
            .user_token_account(self.user_ata)
            .token2022_program(TOKEN_2022_PROGRAM_ID)
            .event_authority(event_authority)
            .quantity(self.quantity);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone(), self.user.insecure_clone()],
            name: "UsePoints",
        }
    }
}

pub struct UsePointsSetupBuilder<'a> {
    ctx: &'a mut TestContext,
    quantity: u64,
    issue_quantity: u64,
}

impl<'a> UsePointsSetupBuilder<'a> {
    fn new(ctx: &'a mut TestContext) -> Self {
        Self { ctx, quantity: 500, issue_quantity: DEFAULT_ISSUE_QUANTITY }
    }

    pub fn quantity(mut self, quantity: u64) -> Self {
        self.quantity = quantity;
        self
    }

    pub fn issue_quantity(mut self, issue_quantity: u64) -> Self {
        self.issue_quantity = issue_quantity;
        self
    }

    pub fn build(self) -> UsePointsSetup {
        // Create config and issue points
        let init_setup = crate::fixtures::InitPointsSetup::builder(self.ctx).build();
        let init_ix = init_setup.build_instruction(self.ctx);
        init_ix.send_expect_success(self.ctx);

        let user = self.ctx.create_funded_keypair();
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
            quantity: self.issue_quantity,
        };
        let issue_ix = issue_setup.build_instruction(self.ctx);
        issue_ix.send_expect_success(self.ctx);

        UsePointsSetup {
            authority: init_setup.authority,
            user,
            points_config_pda: init_setup.points_config_pda,
            points_mint_pda: init_setup.points_mint_pda,
            user_ata,
            quantity: self.quantity,
            issued_quantity: self.issue_quantity,
        }
    }
}

pub struct UsePointsFixture;

impl InstructionTestFixture for UsePointsFixture {
    const INSTRUCTION_NAME: &'static str = "UsePoints";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = UsePointsSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    /// 0: authority, 1: user
    fn required_signers() -> &'static [usize] {
        &[0, 1]
    }

    /// 3: points_mint, 4: user_token_account
    fn required_writable() -> &'static [usize] {
        &[3, 4]
    }

    fn system_program_index() -> Option<usize> {
        None
    }

    fn current_program_index() -> Option<usize> {
        Some(7)
    }

    fn data_len() -> usize {
        1 + 8 // discriminator + quantity
    }
}
