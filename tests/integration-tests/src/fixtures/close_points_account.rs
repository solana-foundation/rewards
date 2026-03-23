use rewards_program_client::instructions::ClosePointsAccountBuilder;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

use crate::fixtures::IssuePointsSetup;
use crate::utils::{
    find_event_authority_pda, find_user_points_pda, InstructionTestFixture, TestContext, TestInstruction,
};

pub struct ClosePointsAccountSetup {
    pub authority: Keypair,
    pub user: Keypair,
    pub points_config_pda: Pubkey,
    pub user_points_pda: Pubkey,
    pub destination: Pubkey,
}

impl ClosePointsAccountSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        // Create config, issue points, then use all points to zero balance
        let init_setup = crate::fixtures::InitPointsSetup::builder(ctx).build();
        let init_ix = init_setup.build_instruction(ctx);
        init_ix.send_expect_success(ctx);

        let user = ctx.create_funded_keypair();
        let (user_points_pda, user_points_bump) = find_user_points_pda(&init_setup.points_config_pda, &user.pubkey());

        // Issue points
        let issue_setup = IssuePointsSetup {
            authority: init_setup.authority.insecure_clone(),
            points_config_pda: init_setup.points_config_pda,
            user: user.pubkey(),
            user_points_pda,
            user_points_bump,
            quantity: 100,
        };
        let issue_ix = issue_setup.build_instruction(ctx);
        issue_ix.send_expect_success(ctx);

        // Use all points to get balance to zero
        let mut use_builder = rewards_program_client::instructions::UsePointsBuilder::new();
        let (event_authority, _) = find_event_authority_pda();
        use_builder
            .authority(init_setup.authority.pubkey())
            .user(user.pubkey())
            .points_config(init_setup.points_config_pda)
            .user_points_account(user_points_pda)
            .event_authority(event_authority)
            .quantity(100);

        let use_ix = TestInstruction {
            instruction: use_builder.instruction(),
            signers: vec![init_setup.authority.insecure_clone(), user.insecure_clone()],
            name: "UsePoints",
        };
        use_ix.send_expect_success(ctx);

        let destination = ctx.create_funded_keypair();

        ClosePointsAccountSetup {
            authority: init_setup.authority,
            user,
            points_config_pda: init_setup.points_config_pda,
            user_points_pda,
            destination: destination.pubkey(),
        }
    }

    pub fn build_instruction(&self, _ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = ClosePointsAccountBuilder::new();
        builder
            .authority(self.authority.pubkey())
            .points_config(self.points_config_pda)
            .user(self.user.pubkey())
            .user_points_account(self.user_points_pda)
            .destination(self.destination)
            .event_authority(event_authority);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone()],
            name: "ClosePointsAccount",
        }
    }
}

pub struct ClosePointsAccountFixture;

impl InstructionTestFixture for ClosePointsAccountFixture {
    const INSTRUCTION_NAME: &'static str = "ClosePointsAccount";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = ClosePointsAccountSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    /// 0: authority
    fn required_signers() -> &'static [usize] {
        &[0]
    }

    /// 3: user_points_account, 4: destination
    fn required_writable() -> &'static [usize] {
        &[3, 4]
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
