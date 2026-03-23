use rewards_program_client::instructions::TransferPointsBuilder;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

use crate::fixtures::{IssuePointsSetup, DEFAULT_ISSUE_QUANTITY};
use crate::utils::{
    find_event_authority_pda, find_user_points_pda, InstructionTestFixture, TestContext, TestInstruction,
};

pub struct TransferPointsSetup {
    pub authority: Keypair,
    pub from_user: Keypair,
    pub to_user: Pubkey,
    pub points_config_pda: Pubkey,
    pub from_user_points_pda: Pubkey,
    pub to_user_points_pda: Pubkey,
    pub to_user_points_bump: u8,
    pub quantity: u64,
    pub issued_quantity: u64,
}

impl TransferPointsSetup {
    pub fn builder(ctx: &mut TestContext) -> TransferPointsSetupBuilder<'_> {
        TransferPointsSetupBuilder::new(ctx)
    }

    pub fn new(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).build()
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = TransferPointsBuilder::new();
        builder
            .payer(ctx.payer.pubkey())
            .authority(self.authority.pubkey())
            .from_user(self.from_user.pubkey())
            .points_config(self.points_config_pda)
            .from_user_points(self.from_user_points_pda)
            .to_user(self.to_user)
            .to_user_points(self.to_user_points_pda)
            .event_authority(event_authority)
            .to_user_points_bump(self.to_user_points_bump)
            .quantity(self.quantity);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone(), self.from_user.insecure_clone()],
            name: "TransferPoints",
        }
    }
}

pub struct TransferPointsSetupBuilder<'a> {
    ctx: &'a mut TestContext,
    quantity: u64,
    issue_quantity: u64,
}

impl<'a> TransferPointsSetupBuilder<'a> {
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

    pub fn build(self) -> TransferPointsSetup {
        // Create config with transferable=1
        let init_setup = crate::fixtures::InitPointsSetup::builder(self.ctx).transferable(1).build();
        let init_ix = init_setup.build_instruction(self.ctx);
        init_ix.send_expect_success(self.ctx);

        let from_user = self.ctx.create_funded_keypair();
        let (from_user_points_pda, from_user_points_bump) =
            find_user_points_pda(&init_setup.points_config_pda, &from_user.pubkey());

        // Issue points to from_user
        let issue_setup = IssuePointsSetup {
            authority: init_setup.authority.insecure_clone(),
            points_config_pda: init_setup.points_config_pda,
            user: from_user.pubkey(),
            user_points_pda: from_user_points_pda,
            user_points_bump: from_user_points_bump,
            quantity: self.issue_quantity,
        };
        let issue_ix = issue_setup.build_instruction(self.ctx);
        issue_ix.send_expect_success(self.ctx);

        let to_user = Keypair::new();
        let (to_user_points_pda, to_user_points_bump) =
            find_user_points_pda(&init_setup.points_config_pda, &to_user.pubkey());

        TransferPointsSetup {
            authority: init_setup.authority,
            from_user,
            to_user: to_user.pubkey(),
            points_config_pda: init_setup.points_config_pda,
            from_user_points_pda,
            to_user_points_pda,
            to_user_points_bump,
            quantity: self.quantity,
            issued_quantity: self.issue_quantity,
        }
    }
}

pub struct TransferPointsFixture;

impl InstructionTestFixture for TransferPointsFixture {
    const INSTRUCTION_NAME: &'static str = "TransferPoints";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = TransferPointsSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    /// 0: payer, 1: authority, 2: from_user
    fn required_signers() -> &'static [usize] {
        &[0, 1, 2]
    }

    /// 0: payer, 4: from_user_points, 6: to_user_points
    fn required_writable() -> &'static [usize] {
        &[0, 4, 6]
    }

    fn system_program_index() -> Option<usize> {
        Some(7)
    }

    fn current_program_index() -> Option<usize> {
        Some(9)
    }

    fn data_len() -> usize {
        1 + 1 + 8 // discriminator + to_user_points_bump + quantity
    }
}
