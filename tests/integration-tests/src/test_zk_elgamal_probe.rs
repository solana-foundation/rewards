/// Probe: does LiteSVM's ZK ElGamal proof program work end-to-end?
///
/// `VerifyPubkeyValidity` is the cheapest instruction available — it takes a 96-byte proof
/// inline (no accounts, no context-state creation) and just verifies the math.  If this test
/// passes, the rest of the confidential-transfer test suite can be built on top of LiteSVM
/// without any external cluster.
///
/// Notes on setup:
///  - `LiteSVM::new()` uses `FeatureSet::default()`, which mirrors mainnet and does NOT include
///    `zk_elgamal_proof_program_enabled`.
///  - `FeatureSet::all_enabled()` activates every known feature flag.
///  - `with_builtins()` reads that feature set and loads the ZK ElGamal proof program.
///  - The standard `TestContext` never calls either, so the program is absent there.
use agave_feature_set::FeatureSet;
use litesvm::LiteSVM;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_zk_sdk::{
    encryption::elgamal::ElGamalKeypair,
    zk_elgamal_proof_program::{
        id as zk_program_id, instruction::ProofInstruction, proof_data::pubkey_validity::PubkeyValidityProofData,
    },
};

fn make_svm_with_zk() -> LiteSVM {
    LiteSVM::new().with_feature_set(FeatureSet::all_enabled()).with_builtins().with_sysvars()
}

#[test]
fn test_zk_elgamal_proof_program_is_registered() {
    let svm = make_svm_with_zk();
    let account = svm.get_account(&zk_program_id());
    assert!(account.is_some(), "ZK ElGamal proof program not registered in LiteSVM");
    assert!(account.unwrap().executable, "ZK ElGamal proof program account is not executable");
}

#[test]
fn test_verify_pubkey_validity_succeeds() {
    let mut svm = make_svm_with_zk();

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let elgamal_kp = ElGamalKeypair::new_rand();
    let proof_data = PubkeyValidityProofData::new(&elgamal_kp).expect("proof generation failed");

    // Inline proof, no context-state account — zero accounts required.
    let ix = ProofInstruction::VerifyPubkeyValidity.encode_verify_proof(None, &proof_data);

    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    svm.send_transaction(tx).expect("VerifyPubkeyValidity should succeed in LiteSVM");
}

#[test]
fn test_verify_pubkey_validity_bad_proof_fails() {
    let mut svm = make_svm_with_zk();

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let elgamal_kp = ElGamalKeypair::new_rand();
    let proof_data = PubkeyValidityProofData::new(&elgamal_kp).expect("proof generation failed");

    let mut ix = ProofInstruction::VerifyPubkeyValidity.encode_verify_proof(None, &proof_data);
    // Corrupt a proof byte (data[0] is the discriminant, data[1..] is the proof struct).
    ix.data[1] ^= 0xff;

    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    assert!(svm.send_transaction(tx).is_err(), "corrupted proof should be rejected");
}

#[test]
fn test_standard_test_context_has_zk_program() {
    use crate::utils::TestContext;
    let ctx = TestContext::new();
    // FeatureSet::default() in Agave 3.x already activates zk_elgamal_proof_program_enabled,
    // so the standard TestContext is sufficient for confidential-transfer tests — no extra
    // cluster or special LiteSVM setup needed.
    let account = ctx.svm.get_account(&zk_program_id());
    assert!(account.is_some(), "ZK ElGamal proof program missing from standard TestContext");
    assert!(account.unwrap().executable, "ZK ElGamal proof program not executable in standard TestContext");
}
