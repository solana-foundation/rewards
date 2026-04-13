use rewards_program_client::instructions::CloseDirectDistributionBuilder;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

use crate::fixtures::CreateDirectDistributionSetup;
use crate::utils::{find_event_authority_pda, InstructionTestFixture, TestContext, TestInstruction};

pub struct CloseDirectDistributionSetup {
    pub authority: Keypair,
    pub distribution_pda: Pubkey,
    pub tombstone_pda: Pubkey,
    pub mint: Pubkey,
    pub distribution_vault: Pubkey,
    pub authority_token_account: Pubkey,
    pub token_program: Pubkey,
}

impl CloseDirectDistributionSetup {
    pub fn builder(ctx: &mut TestContext) -> CloseDirectDistributionSetupBuilder<'_> {
        CloseDirectDistributionSetupBuilder::new(ctx)
    }

    pub fn new(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).build()
    }

    pub fn new_token_2022(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).token_2022().build()
    }

    pub fn from_distribution_setup(ctx: &mut TestContext, distribution_setup: &CreateDirectDistributionSetup) -> Self {
        let instruction = distribution_setup.build_instruction(ctx);
        instruction.send_expect_success(ctx);

        let authority_token_account = if distribution_setup.token_program == TOKEN_2022_PROGRAM_ID {
            ctx.create_token_2022_account(&distribution_setup.authority.pubkey(), &distribution_setup.mint.pubkey())
        } else {
            ctx.create_token_account(&distribution_setup.authority.pubkey(), &distribution_setup.mint.pubkey())
        };

        Self {
            authority: distribution_setup.authority.insecure_clone(),
            distribution_pda: distribution_setup.distribution_pda,
            tombstone_pda: distribution_setup.tombstone_pda,
            mint: distribution_setup.mint.pubkey(),
            distribution_vault: distribution_setup.distribution_vault,
            authority_token_account,
            token_program: distribution_setup.token_program,
        }
    }

    pub fn build_instruction(&self, _ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = CloseDirectDistributionBuilder::new();
        builder
            .authority(self.authority.pubkey())
            .distribution(self.distribution_pda)
            .tombstone(self.tombstone_pda)
            .mint(self.mint)
            .distribution_vault(self.distribution_vault)
            .authority_token_account(self.authority_token_account)
            .system_program(SYSTEM_PROGRAM_ID)
            .token_program(self.token_program)
            .event_authority(event_authority);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone()],
            name: "CloseDirectDistribution",
        }
    }

    pub fn build_instruction_with_wrong_authority(
        &self,
        _ctx: &TestContext,
        wrong_authority: &Keypair,
        wrong_token_account: Pubkey,
    ) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = CloseDirectDistributionBuilder::new();
        builder
            .authority(wrong_authority.pubkey())
            .distribution(self.distribution_pda)
            .tombstone(self.tombstone_pda)
            .mint(self.mint)
            .distribution_vault(self.distribution_vault)
            .authority_token_account(wrong_token_account)
            .system_program(SYSTEM_PROGRAM_ID)
            .token_program(self.token_program)
            .event_authority(event_authority);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![wrong_authority.insecure_clone()],
            name: "CloseDirectDistribution",
        }
    }
}

pub struct CloseDirectDistributionSetupBuilder<'a> {
    ctx: &'a mut TestContext,
    token_program: Pubkey,
}

impl<'a> CloseDirectDistributionSetupBuilder<'a> {
    fn new(ctx: &'a mut TestContext) -> Self {
        Self { ctx, token_program: TOKEN_PROGRAM_ID }
    }

    pub fn token_2022(mut self) -> Self {
        self.token_program = TOKEN_2022_PROGRAM_ID;
        self
    }

    pub fn token_program(mut self, program: Pubkey) -> Self {
        self.token_program = program;
        self
    }

    pub fn build(self) -> CloseDirectDistributionSetup {
        let mut distribution_builder = CreateDirectDistributionSetup::builder(self.ctx);
        if self.token_program == TOKEN_2022_PROGRAM_ID {
            distribution_builder = distribution_builder.token_2022();
        }
        let distribution_setup = distribution_builder.build();
        let create_ix = distribution_setup.build_instruction(self.ctx);
        create_ix.send_expect_success(self.ctx);

        let authority_token_account = if self.token_program == TOKEN_2022_PROGRAM_ID {
            self.ctx
                .create_token_2022_account(&distribution_setup.authority.pubkey(), &distribution_setup.mint.pubkey())
        } else {
            self.ctx.create_token_account(&distribution_setup.authority.pubkey(), &distribution_setup.mint.pubkey())
        };

        CloseDirectDistributionSetup {
            authority: distribution_setup.authority,
            distribution_pda: distribution_setup.distribution_pda,
            tombstone_pda: distribution_setup.tombstone_pda,
            mint: distribution_setup.mint.pubkey(),
            distribution_vault: distribution_setup.distribution_vault,
            authority_token_account,
            token_program: self.token_program,
        }
    }
}

pub struct CloseDirectDistributionFixture;

impl InstructionTestFixture for CloseDirectDistributionFixture {
    const INSTRUCTION_NAME: &'static str = "CloseDirectDistribution";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = CloseDirectDistributionSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    /// Account indices that must be signers:
    /// 0: authority
    fn required_signers() -> &'static [usize] {
        &[0]
    }

    /// Account indices that must be writable:
    /// 0: authority
    /// 1: distribution
    /// 2: tombstone
    /// 4: distribution_vault
    /// 5: authority_token_account
    fn required_writable() -> &'static [usize] {
        &[0, 1, 2, 4, 5]
    }

    fn system_program_index() -> Option<usize> {
        Some(6)
    }

    fn current_program_index() -> Option<usize> {
        Some(9)
    }

    fn data_len() -> usize {
        1 // discriminator only
    }
}
