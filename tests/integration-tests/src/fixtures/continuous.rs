use rewards_program_client::instructions::{
    ClaimContinuousBuilder, ClaimContinuousMerkleBuilder, CloseContinuousPoolBuilder, ContinuousOptInBuilder,
    ContinuousOptOutBuilder, CreateContinuousPoolBuilder, DistributeContinuousRewardBuilder,
    RevokeContinuousUserBuilder, SetContinuousBalanceBuilder, SetContinuousMerkleRootBuilder,
    SyncContinuousBalanceBuilder,
};
use rewards_program_client::types::{BalanceSource, RevokeMode};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

use crate::utils::{
    find_event_authority_pda, find_merkle_claim_pda, find_revocation_pda, find_reward_pool_pda,
    find_user_reward_account_pda, ContinuousMerkleLeaf, ContinuousMerkleTree, InstructionTestFixture, TestContext,
    TestInstruction,
};

pub const DEFAULT_TRACKED_BALANCE: u64 = 1_000_000;
pub const DEFAULT_REWARD_AMOUNT: u64 = 100_000;

pub struct CreateContinuousPoolSetup {
    pub authority: Keypair,
    pub seed: Keypair,
    pub tracked_mint: Keypair,
    pub reward_mint: Keypair,
    pub reward_vault: Pubkey,
    pub reward_pool_pda: Pubkey,
    pub bump: u8,
    pub balance_source: BalanceSource,
    pub revocable: u8,
    pub clawback_ts: i64,
    pub reward_token_program: Pubkey,
}

impl CreateContinuousPoolSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        Self::new_with_balance_source(ctx, BalanceSource::OnChain)
    }

    pub fn new_token_2022(ctx: &mut TestContext) -> Self {
        Self::new_with_balance_source_and_program(ctx, BalanceSource::OnChain, TOKEN_2022_PROGRAM_ID)
    }

    pub fn new_authority_set(ctx: &mut TestContext) -> Self {
        Self::new_with_balance_source(ctx, BalanceSource::AuthoritySet)
    }

    fn new_with_balance_source(ctx: &mut TestContext, balance_source: BalanceSource) -> Self {
        Self::new_with_balance_source_and_program(ctx, balance_source, TOKEN_PROGRAM_ID)
    }

    fn new_with_balance_source_and_program(
        ctx: &mut TestContext,
        balance_source: BalanceSource,
        reward_token_program: Pubkey,
    ) -> Self {
        let authority = ctx.create_funded_keypair();
        let seed = Keypair::new();
        let tracked_mint = Keypair::new();
        let reward_mint = Keypair::new();

        ctx.create_mint_for_program(&tracked_mint, &ctx.payer.pubkey(), 6, &TOKEN_PROGRAM_ID);
        ctx.create_mint_for_program(&reward_mint, &ctx.payer.pubkey(), 6, &reward_token_program);

        let (reward_pool_pda, bump) =
            find_reward_pool_pda(&reward_mint.pubkey(), &tracked_mint.pubkey(), &authority.pubkey(), &seed.pubkey());
        let reward_vault = ctx.create_ata_for_program(&reward_pool_pda, &reward_mint.pubkey(), &reward_token_program);

        CreateContinuousPoolSetup {
            authority,
            seed,
            tracked_mint,
            reward_mint,
            reward_vault,
            reward_pool_pda,
            bump,
            balance_source,
            revocable: 0,
            clawback_ts: 0,
            reward_token_program,
        }
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = CreateContinuousPoolBuilder::new();
        builder
            .payer(ctx.payer.pubkey())
            .authority(self.authority.pubkey())
            .seed(self.seed.pubkey())
            .reward_pool(self.reward_pool_pda)
            .tracked_mint(self.tracked_mint.pubkey())
            .reward_mint(self.reward_mint.pubkey())
            .reward_vault(self.reward_vault)
            .reward_token_program(self.reward_token_program)
            .event_authority(event_authority)
            .bump(self.bump)
            .balance_source(self.balance_source)
            .revocable(self.revocable)
            .clawback_ts(self.clawback_ts);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone(), self.seed.insecure_clone()],
            name: "CreateContinuousPool",
        }
    }
}

pub struct CreateContinuousPoolFixture;

impl InstructionTestFixture for CreateContinuousPoolFixture {
    const INSTRUCTION_NAME: &'static str = "CreateContinuousPool";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = CreateContinuousPoolSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    fn required_signers() -> &'static [usize] {
        &[0, 1, 2] // payer, authority, seeds
    }

    fn required_writable() -> &'static [usize] {
        &[0, 3, 6] // payer, reward_pool, reward_vault
    }

    fn system_program_index() -> Option<usize> {
        Some(7)
    }

    fn current_program_index() -> Option<usize> {
        Some(11)
    }

    fn data_len() -> usize {
        1 + 1 + 1 + 1 + 8 // discriminator + bump + balance_source + revocable + clawback_ts
    }
}

pub struct ContinuousOptInSetup {
    pub pool_setup: CreateContinuousPoolSetup,
    pub user: Keypair,
    pub user_tracked_token_account: Pubkey,
    pub user_reward_pda: Pubkey,
    pub user_reward_bump: u8,
    pub initial_balance: u64,
}

impl ContinuousOptInSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        Self::new_with_balance(ctx, DEFAULT_TRACKED_BALANCE, 0)
    }

    pub fn new_authority_set(ctx: &mut TestContext) -> Self {
        Self::new_authority_set_inner(ctx)
    }

    fn new_with_balance(ctx: &mut TestContext, balance: u64, balance_source: u8) -> Self {
        let pool_setup = if balance_source == 0 {
            CreateContinuousPoolSetup::new(ctx)
        } else {
            CreateContinuousPoolSetup::new_authority_set(ctx)
        };

        pool_setup.build_instruction(ctx).send_expect_success(ctx);

        let user = ctx.create_funded_keypair();
        let user_tracked_token_account =
            ctx.create_token_account_with_balance(&user.pubkey(), &pool_setup.tracked_mint.pubkey(), balance);

        let (user_reward_pda, user_reward_bump) =
            find_user_reward_account_pda(&pool_setup.reward_pool_pda, &user.pubkey());

        ContinuousOptInSetup {
            pool_setup,
            user,
            user_tracked_token_account,
            user_reward_pda,
            user_reward_bump,
            initial_balance: balance,
        }
    }

    fn new_authority_set_inner(ctx: &mut TestContext) -> Self {
        Self::new_with_balance(ctx, 0, 1)
    }

    pub fn new_from_pool(ctx: &mut TestContext, pool_setup: CreateContinuousPoolSetup) -> Self {
        let user = ctx.create_funded_keypair();
        let user_tracked_token_account = ctx.create_token_account_with_balance(
            &user.pubkey(),
            &pool_setup.tracked_mint.pubkey(),
            DEFAULT_TRACKED_BALANCE,
        );

        let (user_reward_pda, user_reward_bump) =
            find_user_reward_account_pda(&pool_setup.reward_pool_pda, &user.pubkey());

        ContinuousOptInSetup {
            pool_setup,
            user,
            user_tracked_token_account,
            user_reward_pda,
            user_reward_bump,
            initial_balance: DEFAULT_TRACKED_BALANCE,
        }
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();
        let (revocation_pda, _) = find_revocation_pda(&self.pool_setup.reward_pool_pda, &self.user.pubkey());

        let mut builder = ContinuousOptInBuilder::new();
        builder
            .payer(ctx.payer.pubkey())
            .user(self.user.pubkey())
            .reward_pool(self.pool_setup.reward_pool_pda)
            .user_reward_account(self.user_reward_pda)
            .revocation_marker(revocation_pda)
            .user_tracked_token_account(self.user_tracked_token_account)
            .tracked_mint(self.pool_setup.tracked_mint.pubkey())
            .tracked_token_program(TOKEN_PROGRAM_ID)
            .event_authority(event_authority)
            .bump(self.user_reward_bump);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.user.insecure_clone()],
            name: "ContinuousOptIn",
        }
    }
}

pub struct ContinuousOptInFixture;

impl InstructionTestFixture for ContinuousOptInFixture {
    const INSTRUCTION_NAME: &'static str = "ContinuousOptIn";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = ContinuousOptInSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    fn required_signers() -> &'static [usize] {
        &[0, 1] // payer, user
    }

    fn required_writable() -> &'static [usize] {
        &[0, 2, 3] // payer, reward_pool, user_reward_account
    }

    fn system_program_index() -> Option<usize> {
        Some(7)
    }

    fn current_program_index() -> Option<usize> {
        Some(10)
    }

    fn data_len() -> usize {
        1 + 1 // discriminator + bump
    }
}

pub struct DistributeContinuousRewardSetup {
    pub opt_in_setup: ContinuousOptInSetup,
    pub authority_token_account: Pubkey,
    pub amount: u64,
}

impl DistributeContinuousRewardSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        Self::new_with_amount(ctx, DEFAULT_REWARD_AMOUNT)
    }

    pub fn new_with_amount(ctx: &mut TestContext, amount: u64) -> Self {
        let opt_in_setup = ContinuousOptInSetup::new(ctx);
        opt_in_setup.build_instruction(ctx).send_expect_success(ctx);

        let authority_token_account = ctx.create_token_account_with_balance(
            &opt_in_setup.pool_setup.authority.pubkey(),
            &opt_in_setup.pool_setup.reward_mint.pubkey(),
            amount * 10,
        );

        DistributeContinuousRewardSetup { opt_in_setup, authority_token_account, amount }
    }

    pub fn build_instruction(&self, _ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();
        let pool = &self.opt_in_setup.pool_setup;

        let mut builder = DistributeContinuousRewardBuilder::new();
        builder
            .authority(pool.authority.pubkey())
            .reward_pool(pool.reward_pool_pda)
            .reward_mint(pool.reward_mint.pubkey())
            .reward_vault(pool.reward_vault)
            .authority_token_account(self.authority_token_account)
            .reward_token_program(pool.reward_token_program)
            .event_authority(event_authority)
            .amount(self.amount);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![pool.authority.insecure_clone()],
            name: "DistributeContinuousReward",
        }
    }
}

pub struct DistributeContinuousRewardFixture;

impl InstructionTestFixture for DistributeContinuousRewardFixture {
    const INSTRUCTION_NAME: &'static str = "DistributeContinuousReward";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = DistributeContinuousRewardSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    fn required_signers() -> &'static [usize] {
        &[0] // authority
    }

    fn required_writable() -> &'static [usize] {
        &[1, 3, 4] // reward_pool, reward_vault, authority_token_account
    }

    fn current_program_index() -> Option<usize> {
        Some(7)
    }

    fn data_len() -> usize {
        1 + 8 // discriminator + amount
    }
}

pub fn build_claim_continuous_instruction(
    _ctx: &TestContext,
    pool_setup: &CreateContinuousPoolSetup,
    user: &Keypair,
    user_reward_pda: &Pubkey,
    user_tracked_token_account: &Pubkey,
    user_reward_token_account: &Pubkey,
    amount: u64,
) -> TestInstruction {
    let (event_authority, _) = find_event_authority_pda();

    let mut builder = ClaimContinuousBuilder::new();
    builder
        .user(user.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .user_reward_account(*user_reward_pda)
        .user_tracked_token_account(*user_tracked_token_account)
        .reward_vault(pool_setup.reward_vault)
        .user_reward_token_account(*user_reward_token_account)
        .tracked_mint(pool_setup.tracked_mint.pubkey())
        .reward_mint(pool_setup.reward_mint.pubkey())
        .tracked_token_program(TOKEN_PROGRAM_ID)
        .reward_token_program(pool_setup.reward_token_program)
        .event_authority(event_authority)
        .amount(amount);

    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![user.insecure_clone()],
        name: "ClaimContinuous",
    }
}

pub fn build_set_continuous_merkle_root_instruction(
    pool_setup: &CreateContinuousPoolSetup,
    merkle_root: [u8; 32],
    epoch: u64,
) -> TestInstruction {
    let mut builder = SetContinuousMerkleRootBuilder::new();
    builder
        .authority(pool_setup.authority.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .merkle_root(merkle_root)
        .epoch(epoch);

    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![pool_setup.authority.insecure_clone()],
        name: "SetContinuousMerkleRoot",
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_claim_continuous_merkle_instruction(
    ctx: &TestContext,
    pool_setup: &CreateContinuousPoolSetup,
    user: &Keypair,
    claim_pda: Pubkey,
    claim_bump: u8,
    user_reward_token_account: Pubkey,
    epoch: u64,
    cumulative_amount: u64,
    amount: u64,
    proof: Vec<[u8; 32]>,
) -> TestInstruction {
    let (event_authority, _) = find_event_authority_pda();
    let (revocation_pda, _) = find_revocation_pda(&pool_setup.reward_pool_pda, &user.pubkey());

    let mut builder = ClaimContinuousMerkleBuilder::new();
    builder
        .payer(ctx.payer.pubkey())
        .user(user.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .claim_account(claim_pda)
        .revocation_marker(revocation_pda)
        .reward_mint(pool_setup.reward_mint.pubkey())
        .reward_vault(pool_setup.reward_vault)
        .user_reward_token_account(user_reward_token_account)
        .reward_token_program(pool_setup.reward_token_program)
        .event_authority(event_authority)
        .claim_bump(claim_bump)
        .epoch(epoch)
        .cumulative_amount(cumulative_amount)
        .amount(amount)
        .proof(proof);

    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![user.insecure_clone()],
        name: "ClaimContinuousMerkle",
    }
}

pub fn build_opt_out_instruction(
    _ctx: &TestContext,
    pool_setup: &CreateContinuousPoolSetup,
    user: &Keypair,
    user_reward_pda: &Pubkey,
    user_tracked_token_account: &Pubkey,
    user_reward_token_account: &Pubkey,
) -> TestInstruction {
    let (event_authority, _) = find_event_authority_pda();

    let mut builder = ContinuousOptOutBuilder::new();
    builder
        .user(user.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .user_reward_account(*user_reward_pda)
        .user_tracked_token_account(*user_tracked_token_account)
        .reward_vault(pool_setup.reward_vault)
        .user_reward_token_account(*user_reward_token_account)
        .tracked_mint(pool_setup.tracked_mint.pubkey())
        .reward_mint(pool_setup.reward_mint.pubkey())
        .tracked_token_program(TOKEN_PROGRAM_ID)
        .reward_token_program(pool_setup.reward_token_program)
        .event_authority(event_authority);

    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![user.insecure_clone()],
        name: "ContinuousOptOut",
    }
}

pub fn build_sync_balance_instruction(
    pool_setup: &CreateContinuousPoolSetup,
    user: &Pubkey,
    user_reward_pda: &Pubkey,
    user_tracked_token_account: &Pubkey,
) -> TestInstruction {
    let (event_authority, _) = find_event_authority_pda();

    let mut builder = SyncContinuousBalanceBuilder::new();
    builder
        .reward_pool(pool_setup.reward_pool_pda)
        .user_reward_account(*user_reward_pda)
        .user(*user)
        .user_tracked_token_account(*user_tracked_token_account)
        .tracked_mint(pool_setup.tracked_mint.pubkey())
        .tracked_token_program(TOKEN_PROGRAM_ID)
        .event_authority(event_authority);

    TestInstruction { instruction: builder.instruction(), signers: vec![], name: "SyncContinuousBalance" }
}

pub fn build_set_balance_instruction(
    pool_setup: &CreateContinuousPoolSetup,
    user: &Pubkey,
    user_reward_pda: &Pubkey,
    balance: u64,
) -> TestInstruction {
    let (event_authority, _) = find_event_authority_pda();

    let mut builder = SetContinuousBalanceBuilder::new();
    builder
        .authority(pool_setup.authority.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .user_reward_account(*user_reward_pda)
        .user(*user)
        .event_authority(event_authority)
        .balance(balance);

    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![pool_setup.authority.insecure_clone()],
        name: "SetContinuousBalance",
    }
}

pub fn build_close_reward_pool_instruction(
    _ctx: &TestContext,
    pool_setup: &CreateContinuousPoolSetup,
    authority_token_account: &Pubkey,
) -> TestInstruction {
    let (event_authority, _) = find_event_authority_pda();

    let mut builder = CloseContinuousPoolBuilder::new();
    builder
        .authority(pool_setup.authority.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .reward_mint(pool_setup.reward_mint.pubkey())
        .reward_vault(pool_setup.reward_vault)
        .authority_token_account(*authority_token_account)
        .reward_token_program(pool_setup.reward_token_program)
        .event_authority(event_authority);

    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![pool_setup.authority.insecure_clone()],
        name: "CloseContinuousPool",
    }
}

pub struct ClaimContinuousSetup {
    pub distribute_setup: DistributeContinuousRewardSetup,
    pub user_reward_token_account: Pubkey,
}

pub struct SetContinuousMerkleRootSetup {
    pub pool_setup: CreateContinuousPoolSetup,
    pub merkle_root: [u8; 32],
    pub epoch: u64,
}

impl SetContinuousMerkleRootSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        let pool_setup = CreateContinuousPoolSetup::new(ctx);
        pool_setup.build_instruction(ctx).send_expect_success(ctx);

        Self { pool_setup, merkle_root: [11u8; 32], epoch: 1 }
    }

    pub fn build_instruction(&self) -> TestInstruction {
        build_set_continuous_merkle_root_instruction(&self.pool_setup, self.merkle_root, self.epoch)
    }
}

pub struct SetContinuousMerkleRootFixture;

impl InstructionTestFixture for SetContinuousMerkleRootFixture {
    const INSTRUCTION_NAME: &'static str = "SetContinuousMerkleRoot";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = SetContinuousMerkleRootSetup::new(ctx);
        setup.build_instruction()
    }

    fn required_signers() -> &'static [usize] {
        &[0] // authority
    }

    fn required_writable() -> &'static [usize] {
        &[1] // reward_pool
    }

    fn current_program_index() -> Option<usize> {
        Some(2)
    }

    fn data_len() -> usize {
        1 + 32 + 8 // discriminator + merkle_root + epoch
    }
}

pub struct ClaimContinuousMerkleSetup {
    pub distribute_setup: DistributeContinuousRewardSetup,
    pub user_reward_token_account: Pubkey,
    pub claim_pda: Pubkey,
    pub claim_bump: u8,
    pub epoch: u64,
    pub cumulative_amount: u64,
    pub proof: Vec<[u8; 32]>,
    pub merkle_tree: ContinuousMerkleTree,
}

impl ClaimContinuousMerkleSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        let distribute_setup = DistributeContinuousRewardSetup::new(ctx);
        distribute_setup.build_instruction(ctx).send_expect_success(ctx);

        let pool_setup = &distribute_setup.opt_in_setup.pool_setup;
        let user = &distribute_setup.opt_in_setup.user;
        let other_claimant = ctx.create_funded_keypair();
        let epoch = 1;
        let cumulative_amount = DEFAULT_REWARD_AMOUNT;

        let leaves = vec![
            ContinuousMerkleLeaf::new(pool_setup.reward_pool_pda, user.pubkey(), epoch, cumulative_amount),
            ContinuousMerkleLeaf::new(
                pool_setup.reward_pool_pda,
                other_claimant.pubkey(),
                epoch,
                cumulative_amount / 2,
            ),
        ];
        let merkle_tree = ContinuousMerkleTree::new(leaves);

        build_set_continuous_merkle_root_instruction(pool_setup, merkle_tree.root, epoch).send_expect_success(ctx);

        let (claim_pda, claim_bump) = find_merkle_claim_pda(&pool_setup.reward_pool_pda, &user.pubkey());
        let proof = merkle_tree.get_proof_for_claimant(&user.pubkey()).unwrap_or_default();
        let user_reward_token_account = ctx.create_token_account(&user.pubkey(), &pool_setup.reward_mint.pubkey());

        Self {
            distribute_setup,
            user_reward_token_account,
            claim_pda,
            claim_bump,
            epoch,
            cumulative_amount,
            proof,
            merkle_tree,
        }
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        self.build_instruction_with_amount(ctx, 0)
    }

    pub fn build_instruction_with_amount(&self, ctx: &TestContext, amount: u64) -> TestInstruction {
        let pool_setup = &self.distribute_setup.opt_in_setup.pool_setup;
        let user = &self.distribute_setup.opt_in_setup.user;
        build_claim_continuous_merkle_instruction(
            ctx,
            pool_setup,
            user,
            self.claim_pda,
            self.claim_bump,
            self.user_reward_token_account,
            self.epoch,
            self.cumulative_amount,
            amount,
            self.proof.clone(),
        )
    }
}

pub struct ClaimContinuousMerkleFixture;

impl InstructionTestFixture for ClaimContinuousMerkleFixture {
    const INSTRUCTION_NAME: &'static str = "ClaimContinuousMerkle";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = ClaimContinuousMerkleSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    fn required_signers() -> &'static [usize] {
        &[0, 1] // payer, user (payer signed by test context)
    }

    fn required_writable() -> &'static [usize] {
        &[0, 2, 3, 6, 7] // payer, reward_pool, claim_account, reward_vault, user_reward_token_account
    }

    fn system_program_index() -> Option<usize> {
        Some(8)
    }

    fn current_program_index() -> Option<usize> {
        Some(11)
    }

    fn data_len() -> usize {
        1 + 1 + 8 + 8 + 8 + 4 + 32 // discriminator + claim_bump + epoch + cumulative + amount + proof_len + one proof hash
    }
}

impl ClaimContinuousSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        let distribute_setup = DistributeContinuousRewardSetup::new(ctx);
        distribute_setup.build_instruction(ctx).send_expect_success(ctx);

        let pool_setup = &distribute_setup.opt_in_setup.pool_setup;
        let user = &distribute_setup.opt_in_setup.user;
        let user_reward_token_account = ctx.create_token_account(&user.pubkey(), &pool_setup.reward_mint.pubkey());

        ClaimContinuousSetup { distribute_setup, user_reward_token_account }
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        let pool_setup = &self.distribute_setup.opt_in_setup.pool_setup;
        let user = &self.distribute_setup.opt_in_setup.user;
        let user_reward_pda = &self.distribute_setup.opt_in_setup.user_reward_pda;
        let user_tracked_ta = &self.distribute_setup.opt_in_setup.user_tracked_token_account;

        build_claim_continuous_instruction(
            ctx,
            pool_setup,
            user,
            user_reward_pda,
            user_tracked_ta,
            &self.user_reward_token_account,
            0,
        )
    }
}

pub struct ClaimContinuousFixture;

impl InstructionTestFixture for ClaimContinuousFixture {
    const INSTRUCTION_NAME: &'static str = "ClaimContinuous";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = ClaimContinuousSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    fn required_signers() -> &'static [usize] {
        &[0] // user
    }

    fn required_writable() -> &'static [usize] {
        &[1, 2, 4, 5] // reward_pool, user_reward_account, reward_vault, user_reward_token_account
    }

    fn current_program_index() -> Option<usize> {
        Some(11)
    }

    fn data_len() -> usize {
        1 + 8 // discriminator + amount
    }
}

pub struct ContinuousOptOutSetup {
    pub distribute_setup: DistributeContinuousRewardSetup,
    pub user_reward_token_account: Pubkey,
}

impl ContinuousOptOutSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        let distribute_setup = DistributeContinuousRewardSetup::new(ctx);
        distribute_setup.build_instruction(ctx).send_expect_success(ctx);

        let pool_setup = &distribute_setup.opt_in_setup.pool_setup;
        let user = &distribute_setup.opt_in_setup.user;
        let user_reward_token_account = ctx.create_token_account(&user.pubkey(), &pool_setup.reward_mint.pubkey());

        ContinuousOptOutSetup { distribute_setup, user_reward_token_account }
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        let pool_setup = &self.distribute_setup.opt_in_setup.pool_setup;
        let user = &self.distribute_setup.opt_in_setup.user;
        let user_reward_pda = &self.distribute_setup.opt_in_setup.user_reward_pda;
        let user_tracked_ta = &self.distribute_setup.opt_in_setup.user_tracked_token_account;

        build_opt_out_instruction(
            ctx,
            pool_setup,
            user,
            user_reward_pda,
            user_tracked_ta,
            &self.user_reward_token_account,
        )
    }
}

pub struct ContinuousOptOutFixture;

impl InstructionTestFixture for ContinuousOptOutFixture {
    const INSTRUCTION_NAME: &'static str = "ContinuousOptOut";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = ContinuousOptOutSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    fn required_signers() -> &'static [usize] {
        &[0] // user
    }

    fn required_writable() -> &'static [usize] {
        &[1, 2, 4, 5] // reward_pool, user_reward_account, reward_vault, user_reward_token_account
    }

    fn current_program_index() -> Option<usize> {
        Some(11)
    }

    fn data_len() -> usize {
        1 // discriminator only (no data)
    }
}

pub struct SyncContinuousBalanceSetup {
    pub opt_in_setup: ContinuousOptInSetup,
}

impl SyncContinuousBalanceSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        let opt_in_setup = ContinuousOptInSetup::new(ctx);
        opt_in_setup.build_instruction(ctx).send_expect_success(ctx);
        SyncContinuousBalanceSetup { opt_in_setup }
    }

    pub fn build_instruction(&self) -> TestInstruction {
        build_sync_balance_instruction(
            &self.opt_in_setup.pool_setup,
            &self.opt_in_setup.user.pubkey(),
            &self.opt_in_setup.user_reward_pda,
            &self.opt_in_setup.user_tracked_token_account,
        )
    }
}

pub struct SyncContinuousBalanceFixture;

impl InstructionTestFixture for SyncContinuousBalanceFixture {
    const INSTRUCTION_NAME: &'static str = "SyncContinuousBalance";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = SyncContinuousBalanceSetup::new(ctx);
        setup.build_instruction()
    }

    fn required_signers() -> &'static [usize] {
        &[] // permissionless
    }

    fn required_writable() -> &'static [usize] {
        &[0, 1] // reward_pool, user_reward_account
    }

    fn current_program_index() -> Option<usize> {
        None
    }

    fn data_len() -> usize {
        1 // discriminator only
    }
}

pub struct SetContinuousBalanceSetup {
    pub opt_in_setup: ContinuousOptInSetup,
}

impl SetContinuousBalanceSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        let opt_in_setup = ContinuousOptInSetup::new_authority_set(ctx);
        opt_in_setup.build_instruction(ctx).send_expect_success(ctx);
        SetContinuousBalanceSetup { opt_in_setup }
    }

    pub fn build_instruction(&self) -> TestInstruction {
        build_set_balance_instruction(
            &self.opt_in_setup.pool_setup,
            &self.opt_in_setup.user.pubkey(),
            &self.opt_in_setup.user_reward_pda,
            500_000,
        )
    }
}

pub struct SetContinuousBalanceFixture;

impl InstructionTestFixture for SetContinuousBalanceFixture {
    const INSTRUCTION_NAME: &'static str = "SetContinuousBalance";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = SetContinuousBalanceSetup::new(ctx);
        setup.build_instruction()
    }

    fn required_signers() -> &'static [usize] {
        &[0] // authority
    }

    fn required_writable() -> &'static [usize] {
        &[1, 2] // reward_pool, user_reward_account
    }

    fn current_program_index() -> Option<usize> {
        Some(5) // rewards_program
    }

    fn data_len() -> usize {
        1 + 8 // discriminator + balance
    }
}

pub struct CloseContinuousPoolSetup {
    pub pool_setup: CreateContinuousPoolSetup,
    pub authority_token_account: Pubkey,
}

impl CloseContinuousPoolSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        let pool_setup = CreateContinuousPoolSetup::new(ctx);
        pool_setup.build_instruction(ctx).send_expect_success(ctx);

        let authority_token_account =
            ctx.create_token_account(&pool_setup.authority.pubkey(), &pool_setup.reward_mint.pubkey());

        CloseContinuousPoolSetup { pool_setup, authority_token_account }
    }

    pub fn new_with_clawback(ctx: &mut TestContext, clawback_ts: i64) -> Self {
        let mut pool_setup = CreateContinuousPoolSetup::new(ctx);
        pool_setup.clawback_ts = clawback_ts;
        pool_setup.build_instruction(ctx).send_expect_success(ctx);

        let authority_token_account =
            ctx.create_token_account(&pool_setup.authority.pubkey(), &pool_setup.reward_mint.pubkey());

        CloseContinuousPoolSetup { pool_setup, authority_token_account }
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        build_close_reward_pool_instruction(ctx, &self.pool_setup, &self.authority_token_account)
    }
}

pub struct CloseContinuousPoolFixture;

impl InstructionTestFixture for CloseContinuousPoolFixture {
    const INSTRUCTION_NAME: &'static str = "CloseContinuousPool";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = CloseContinuousPoolSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    fn required_signers() -> &'static [usize] {
        &[0] // authority
    }

    fn required_writable() -> &'static [usize] {
        &[1, 3, 4] // reward_pool, reward_vault, authority_token_account
    }

    fn current_program_index() -> Option<usize> {
        Some(7)
    }

    fn data_len() -> usize {
        1 // discriminator only
    }
}

pub struct RevokeContinuousUserSetup {
    pub distribute_setup: DistributeContinuousRewardSetup,
    pub user_reward_token_account: Pubkey,
    pub authority_reward_token_account: Pubkey,
}

impl RevokeContinuousUserSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        Self::new_with_revocable(ctx, 3)
    }

    pub fn new_with_revocable(ctx: &mut TestContext, revocable: u8) -> Self {
        let mut pool_setup = CreateContinuousPoolSetup::new(ctx);
        pool_setup.revocable = revocable;
        pool_setup.build_instruction(ctx).send_expect_success(ctx);

        let opt_in_setup = ContinuousOptInSetup::new_from_pool(ctx, pool_setup);
        opt_in_setup.build_instruction(ctx).send_expect_success(ctx);

        let authority_token_account = ctx.create_token_account_with_balance(
            &opt_in_setup.pool_setup.authority.pubkey(),
            &opt_in_setup.pool_setup.reward_mint.pubkey(),
            DEFAULT_REWARD_AMOUNT * 10,
        );

        let distribute_setup =
            DistributeContinuousRewardSetup { opt_in_setup, authority_token_account, amount: DEFAULT_REWARD_AMOUNT };
        distribute_setup.build_instruction(ctx).send_expect_success(ctx);

        let pool_setup = &distribute_setup.opt_in_setup.pool_setup;
        let user = &distribute_setup.opt_in_setup.user;
        let user_reward_token_account = ctx.create_token_account(&user.pubkey(), &pool_setup.reward_mint.pubkey());
        let authority_reward_token_account =
            ctx.create_token_account(&pool_setup.authority.pubkey(), &pool_setup.reward_mint.pubkey());

        RevokeContinuousUserSetup { distribute_setup, user_reward_token_account, authority_reward_token_account }
    }

    pub fn build_instruction(&self, ctx: &TestContext, revoke_mode: RevokeMode) -> TestInstruction {
        let pool_setup = &self.distribute_setup.opt_in_setup.pool_setup;
        let user = &self.distribute_setup.opt_in_setup.user;
        let user_reward_pda = &self.distribute_setup.opt_in_setup.user_reward_pda;
        let user_tracked_ta = &self.distribute_setup.opt_in_setup.user_tracked_token_account;

        build_revoke_user_instruction(
            ctx,
            pool_setup,
            user,
            user_reward_pda,
            user_tracked_ta,
            &self.user_reward_token_account,
            &self.authority_reward_token_account,
            revoke_mode,
        )
    }
}

pub struct RevokeContinuousUserFixture;

impl InstructionTestFixture for RevokeContinuousUserFixture {
    const INSTRUCTION_NAME: &'static str = "RevokeContinuousUser";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = RevokeContinuousUserSetup::new(ctx);
        setup.build_instruction(ctx, RevokeMode::NonVested)
    }

    fn required_signers() -> &'static [usize] {
        &[0] // authority (payer is authority, auto-signed via signers vec)
    }

    fn required_writable() -> &'static [usize] {
        &[2, 3, 4, 6, 8, 9, 10] // reward_pool, user_reward_account, revocation_marker, rent_destination, reward_vault, user_reward_token_account, authority_reward_token_account
    }

    fn system_program_index() -> Option<usize> {
        Some(13)
    }

    fn current_program_index() -> Option<usize> {
        Some(17)
    }

    fn data_len() -> usize {
        1 + 1 // discriminator + revoke_mode
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_revoke_user_instruction(
    ctx: &TestContext,
    pool_setup: &CreateContinuousPoolSetup,
    user: &Keypair,
    user_reward_pda: &Pubkey,
    user_tracked_token_account: &Pubkey,
    user_reward_token_account: &Pubkey,
    authority_reward_token_account: &Pubkey,
    revoke_mode: RevokeMode,
) -> TestInstruction {
    let (event_authority, _) = find_event_authority_pda();
    let (revocation_pda, _) = find_revocation_pda(&pool_setup.reward_pool_pda, &user.pubkey());

    let mut builder = RevokeContinuousUserBuilder::new();
    builder
        .authority(pool_setup.authority.pubkey())
        .payer(ctx.payer.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .user_reward_account(*user_reward_pda)
        .revocation_marker(revocation_pda)
        .user(user.pubkey())
        .rent_destination(user.pubkey())
        .user_tracked_token_account(*user_tracked_token_account)
        .reward_vault(pool_setup.reward_vault)
        .user_reward_token_account(*user_reward_token_account)
        .authority_reward_token_account(*authority_reward_token_account)
        .tracked_mint(pool_setup.tracked_mint.pubkey())
        .reward_mint(pool_setup.reward_mint.pubkey())
        .tracked_token_program(TOKEN_PROGRAM_ID)
        .reward_token_program(pool_setup.reward_token_program)
        .event_authority(event_authority)
        .revoke_mode(revoke_mode);

    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![pool_setup.authority.insecure_clone()],
        name: "RevokeContinuousUser",
    }
}
