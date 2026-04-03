use rewards_program_client::instructions::InitPointsBuilder;
use solana_sdk::signature::{Keypair, Signer};
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;

use crate::utils::{
    find_event_authority_pda, find_points_config_pda, find_points_mint_pda, InstructionTestFixture, TestContext,
    TestInstruction,
};

pub struct InitPointsSetup {
    pub authority: Keypair,
    pub seed: Keypair,
    pub points_config_pda: solana_sdk::pubkey::Pubkey,
    pub points_mint_pda: solana_sdk::pubkey::Pubkey,
    pub bump: u8,
    pub mint_bump: u8,
    pub transferable: u8,
    pub revocable: u8,
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
            .points_mint(self.points_mint_pda)
            .token2022_program(TOKEN_2022_PROGRAM_ID)
            .event_authority(event_authority)
            .bump(self.bump)
            .transferable(self.transferable)
            .revocable(self.revocable)
            .mint_bump(self.mint_bump);

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
}

impl<'a> InitPointsSetupBuilder<'a> {
    fn new(ctx: &'a mut TestContext) -> Self {
        Self { ctx, transferable: 0, revocable: 0 }
    }

    pub fn transferable(mut self, transferable: u8) -> Self {
        self.transferable = transferable;
        self
    }

    pub fn revocable(mut self, revocable: u8) -> Self {
        self.revocable = revocable;
        self
    }

    pub fn build(self) -> InitPointsSetup {
        let authority = self.ctx.create_funded_keypair();
        let seed = Keypair::new();

        let (points_config_pda, bump) = find_points_config_pda(&authority.pubkey(), &seed.pubkey());
        let (points_mint_pda, mint_bump) = find_points_mint_pda(&points_config_pda);

        InitPointsSetup {
            authority,
            seed,
            points_config_pda,
            points_mint_pda,
            bump,
            mint_bump,
            transferable: self.transferable,
            revocable: self.revocable,
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

    /// 0: payer, 3: points_config, 4: points_mint
    fn required_writable() -> &'static [usize] {
        &[0, 3, 4]
    }

    fn system_program_index() -> Option<usize> {
        Some(5)
    }

    fn current_program_index() -> Option<usize> {
        Some(8)
    }

    fn data_len() -> usize {
        1 + 1 + 1 + 1 + 1 // discriminator + bump + transferable + revocable + mint_bump
    }
}
