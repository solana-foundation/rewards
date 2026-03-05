use crate::fixtures::ContinuousOptOutFixture;
use crate::utils::{test_missing_signer, test_not_writable, test_wrong_current_program, TestContext};

// ─── Generic validation tests ───

#[test]
fn test_opt_out_missing_user_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<ContinuousOptOutFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_opt_out_pool_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ContinuousOptOutFixture>(&mut ctx, 1);
}

#[test]
fn test_opt_out_user_reward_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ContinuousOptOutFixture>(&mut ctx, 2);
}

#[test]
fn test_opt_out_vault_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ContinuousOptOutFixture>(&mut ctx, 4);
}

#[test]
fn test_opt_out_user_reward_token_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ContinuousOptOutFixture>(&mut ctx, 5);
}

#[test]
fn test_opt_out_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<ContinuousOptOutFixture>(&mut ctx);
}
