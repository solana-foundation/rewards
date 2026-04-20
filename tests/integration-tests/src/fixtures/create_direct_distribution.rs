use rewards_program_client::instructions::CreateDirectDistributionBuilder;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

use crate::utils::{
    find_direct_distribution_pda, find_event_authority_pda, InstructionTestFixture, TestContext, TestInstruction,
};

pub struct CreateDirectDistributionSetup {
    pub authority: Keypair,
    pub seed: Keypair,
    pub mint: Keypair,
    pub distribution_vault: Pubkey,
    pub distribution_pda: Pubkey,
    pub bump: u8,
    pub token_program: Pubkey,
    pub revocable: u8,
    pub clawback_ts: i64,
}

impl CreateDirectDistributionSetup {
    pub fn builder(ctx: &mut TestContext) -> CreateDirectDistributionSetupBuilder<'_> {
        CreateDirectDistributionSetupBuilder::new(ctx)
    }

    pub fn new(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).build()
    }

    pub fn new_token_2022(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).token_2022().build()
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = CreateDirectDistributionBuilder::new();
        builder
            .payer(ctx.payer.pubkey())
            .authority(self.authority.pubkey())
            .seeds(self.seed.pubkey())
            .distribution(self.distribution_pda)
            .mint(self.mint.pubkey())
            .distribution_vault(self.distribution_vault)
            .token_program(self.token_program)
            .event_authority(event_authority)
            .bump(self.bump)
            .revocable(self.revocable)
            .clawback_ts(self.clawback_ts);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone(), self.seed.insecure_clone()],
            name: "CreateDirectDistribution",
        }
    }
}

pub struct CreateDirectDistributionSetupBuilder<'a> {
    ctx: &'a mut TestContext,
    token_program: Pubkey,
    revocable: u8,
    clawback_ts: i64,
}

impl<'a> CreateDirectDistributionSetupBuilder<'a> {
    fn new(ctx: &'a mut TestContext) -> Self {
        Self { ctx, token_program: TOKEN_PROGRAM_ID, revocable: 0, clawback_ts: 0 }
    }

    pub fn token_2022(mut self) -> Self {
        self.token_program = TOKEN_2022_PROGRAM_ID;
        self
    }

    pub fn token_program(mut self, program: Pubkey) -> Self {
        self.token_program = program;
        self
    }

    pub fn revocable(mut self, revocable: u8) -> Self {
        self.revocable = revocable;
        self
    }

    pub fn clawback_ts(mut self, clawback_ts: i64) -> Self {
        self.clawback_ts = clawback_ts;
        self
    }

    pub fn build(self) -> CreateDirectDistributionSetup {
        let authority = self.ctx.create_funded_keypair();
        let seeds = Keypair::new();
        let mint = Keypair::new();
        let token_program = self.token_program;

        self.ctx.create_mint_for_program(&mint, &self.ctx.payer.pubkey(), 6, &token_program);

        let (distribution_pda, bump) =
            find_direct_distribution_pda(&mint.pubkey(), &authority.pubkey(), &seeds.pubkey());
        let distribution_vault = self.ctx.create_ata_for_program(&distribution_pda, &mint.pubkey(), &token_program);

        CreateDirectDistributionSetup {
            authority,
            seed: seeds,
            mint,
            distribution_vault,
            distribution_pda,
            bump,
            token_program,
            revocable: self.revocable,
            clawback_ts: self.clawback_ts,
        }
    }
}

pub struct CreateDirectDistributionFixture;

impl InstructionTestFixture for CreateDirectDistributionFixture {
    const INSTRUCTION_NAME: &'static str = "CreateDirectDistribution";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = CreateDirectDistributionSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    /// Account indices that must be signers:
    /// 0: payer (handled by TestContext)
    /// 1: authority
    /// 2: seeds
    fn required_signers() -> &'static [usize] {
        &[0, 1, 2]
    }

    /// Account indices that must be writable:
    /// 0: payer (handled by TestContext)
    /// 3: distribution
    /// 5: distribution_vault
    fn required_writable() -> &'static [usize] {
        &[0, 3, 5]
    }

    fn system_program_index() -> Option<usize> {
        Some(6)
    }

    fn current_program_index() -> Option<usize> {
        Some(10)
    }

    fn data_len() -> usize {
        1 + 1 + 1 + 8 // discriminator + bump + revocable + clawback_ts
    }
}
