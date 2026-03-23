use rewards_program_client::instructions::UsePointsBuilder;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

use crate::fixtures::{IssuePointsSetup, DEFAULT_ISSUE_QUANTITY};
use crate::utils::{
    find_event_authority_pda, find_user_points_pda, InstructionTestFixture, TestContext, TestInstruction,
};

pub struct UsePointsSetup {
    pub authority: Keypair,
    pub user: Keypair,
    pub points_config_pda: Pubkey,
    pub user_points_pda: Pubkey,
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
            .user_points_account(self.user_points_pda)
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
        let (user_points_pda, user_points_bump) = find_user_points_pda(&init_setup.points_config_pda, &user.pubkey());

        // Issue points to user
        let issue_setup = IssuePointsSetup {
            authority: init_setup.authority.insecure_clone(),
            points_config_pda: init_setup.points_config_pda,
            user: user.pubkey(),
            user_points_pda,
            user_points_bump,
            quantity: self.issue_quantity,
        };
        let issue_ix = issue_setup.build_instruction(self.ctx);
        issue_ix.send_expect_success(self.ctx);

        UsePointsSetup {
            authority: init_setup.authority,
            user,
            points_config_pda: init_setup.points_config_pda,
            user_points_pda,
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

    /// 2: points_config, 3: user_points_account
    fn required_writable() -> &'static [usize] {
        &[2, 3]
    }

    fn system_program_index() -> Option<usize> {
        None
    }

    fn current_program_index() -> Option<usize> {
        Some(5)
    }

    fn data_len() -> usize {
        1 + 8 // discriminator + quantity
    }
}
