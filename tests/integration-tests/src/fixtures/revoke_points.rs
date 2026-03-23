use rewards_program_client::instructions::RevokePointsBuilder;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

use crate::fixtures::IssuePointsSetup;
use crate::utils::{
    find_event_authority_pda, find_user_points_pda, InstructionTestFixture, TestContext, TestInstruction,
};

pub const DEFAULT_REVOKE_ISSUE_QUANTITY: u64 = 500;

pub struct RevokePointsSetup {
    pub authority: Keypair,
    pub user: Pubkey,
    pub points_config_pda: Pubkey,
    pub user_points_pda: Pubkey,
    pub destination: Pubkey,
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
        let (user_points_pda, user_points_bump) = find_user_points_pda(&init_setup.points_config_pda, &user.pubkey());

        // Issue points to user
        let issue_setup = IssuePointsSetup {
            authority: init_setup.authority.insecure_clone(),
            points_config_pda: init_setup.points_config_pda,
            user: user.pubkey(),
            user_points_pda,
            user_points_bump,
            quantity: issue_quantity,
        };
        let issue_ix = issue_setup.build_instruction(ctx);
        issue_ix.send_expect_success(ctx);

        let destination = ctx.create_funded_keypair();

        RevokePointsSetup {
            authority: init_setup.authority,
            user: user.pubkey(),
            points_config_pda: init_setup.points_config_pda,
            user_points_pda,
            destination: destination.pubkey(),
            issued_quantity: issue_quantity,
        }
    }

    pub fn build_instruction(&self, _ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = RevokePointsBuilder::new();
        builder
            .authority(self.authority.pubkey())
            .points_config(self.points_config_pda)
            .user(self.user)
            .user_points_account(self.user_points_pda)
            .destination(self.destination)
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

    /// 1: points_config, 3: user_points_account, 4: destination
    fn required_writable() -> &'static [usize] {
        &[1, 3, 4]
    }

    fn system_program_index() -> Option<usize> {
        None
    }

    fn current_program_index() -> Option<usize> {
        Some(6)
    }

    fn data_len() -> usize {
        1 // discriminator only
    }
}
