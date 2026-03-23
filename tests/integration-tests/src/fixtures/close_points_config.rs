use rewards_program_client::instructions::ClosePointsConfigBuilder;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

use crate::utils::{find_event_authority_pda, InstructionTestFixture, TestContext, TestInstruction};

pub struct ClosePointsConfigSetup {
    pub authority: Keypair,
    pub points_config_pda: Pubkey,
    pub destination: Pubkey,
}

impl ClosePointsConfigSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        let init_setup = crate::fixtures::InitPointsSetup::builder(ctx).build();
        let init_ix = init_setup.build_instruction(ctx);
        init_ix.send_expect_success(ctx);

        let destination = ctx.create_funded_keypair();

        ClosePointsConfigSetup {
            authority: init_setup.authority,
            points_config_pda: init_setup.points_config_pda,
            destination: destination.pubkey(),
        }
    }

    pub fn build_instruction(&self, _ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = ClosePointsConfigBuilder::new();
        builder
            .authority(self.authority.pubkey())
            .points_config(self.points_config_pda)
            .destination(self.destination)
            .event_authority(event_authority);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone()],
            name: "ClosePointsConfig",
        }
    }
}

pub struct ClosePointsConfigFixture;

impl InstructionTestFixture for ClosePointsConfigFixture {
    const INSTRUCTION_NAME: &'static str = "ClosePointsConfig";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = ClosePointsConfigSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    /// 0: authority
    fn required_signers() -> &'static [usize] {
        &[0]
    }

    /// 1: points_config, 2: destination
    fn required_writable() -> &'static [usize] {
        &[1, 2]
    }

    fn system_program_index() -> Option<usize> {
        None
    }

    fn current_program_index() -> Option<usize> {
        Some(4)
    }

    fn data_len() -> usize {
        1 // discriminator only
    }
}
