use rewards_program_client::instructions::InitPointsBuilder;
use solana_sdk::signature::{Keypair, Signer};

use crate::utils::{
    find_event_authority_pda, find_points_config_pda, InstructionTestFixture, TestContext, TestInstruction,
};

pub struct InitPointsSetup {
    pub authority: Keypair,
    pub seed: Keypair,
    pub points_config_pda: solana_sdk::pubkey::Pubkey,
    pub bump: u8,
    pub transferable: u8,
    pub revocable: u8,
    pub max_supply: u64,
}

impl InitPointsSetup {
    pub fn builder(ctx: &mut TestContext) -> InitPointsSetupBuilder<'_> {
        InitPointsSetupBuilder::new(ctx)
    }

    pub fn new(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).build()
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = InitPointsBuilder::new();
        builder
            .payer(ctx.payer.pubkey())
            .authority(self.authority.pubkey())
            .seeds(self.seed.pubkey())
            .points_config(self.points_config_pda)
            .event_authority(event_authority)
            .bump(self.bump)
            .transferable(self.transferable)
            .revocable(self.revocable)
            .max_supply(self.max_supply);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone(), self.seed.insecure_clone()],
            name: "InitPoints",
        }
    }
}

pub struct InitPointsSetupBuilder<'a> {
    ctx: &'a mut TestContext,
    transferable: u8,
    revocable: u8,
    max_supply: u64,
}

impl<'a> InitPointsSetupBuilder<'a> {
    fn new(ctx: &'a mut TestContext) -> Self {
        Self { ctx, transferable: 0, revocable: 0, max_supply: 0 }
    }

    pub fn transferable(mut self, transferable: u8) -> Self {
        self.transferable = transferable;
        self
    }

    pub fn revocable(mut self, revocable: u8) -> Self {
        self.revocable = revocable;
        self
    }

    pub fn max_supply(mut self, max_supply: u64) -> Self {
        self.max_supply = max_supply;
        self
    }

    pub fn build(self) -> InitPointsSetup {
        let authority = self.ctx.create_funded_keypair();
        let seed = Keypair::new();

        let (points_config_pda, bump) = find_points_config_pda(&authority.pubkey(), &seed.pubkey());

        InitPointsSetup {
            authority,
            seed,
            points_config_pda,
            bump,
            transferable: self.transferable,
            revocable: self.revocable,
            max_supply: self.max_supply,
        }
    }
}

pub struct InitPointsFixture;

impl InstructionTestFixture for InitPointsFixture {
    const INSTRUCTION_NAME: &'static str = "InitPoints";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = InitPointsSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    /// 0: payer, 1: authority, 2: seeds
    fn required_signers() -> &'static [usize] {
        &[0, 1, 2]
    }

    /// 0: payer, 3: points_config
    fn required_writable() -> &'static [usize] {
        &[0, 3]
    }

    fn system_program_index() -> Option<usize> {
        Some(4)
    }

    fn current_program_index() -> Option<usize> {
        Some(6)
    }

    fn data_len() -> usize {
        1 + 1 + 1 + 1 + 8 // discriminator + bump + transferable + revocable + max_supply
    }
}
