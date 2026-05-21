use std::{
    net::TcpListener,
    path::PathBuf,
    thread::sleep,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use crossbeam_channel::{Receiver, Sender};
use solana_client::rpc_request::RpcRequest;
use solana_epoch_info::EpochInfo;
use solana_rpc_client::rpc_client::RpcClient;
use solana_rpc_client_api::config::{RpcSendTransactionConfig, RpcSimulateTransactionConfig, UiTransactionEncoding};
use solana_sdk::{
    account::Account,
    instruction::Instruction,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
};
use surfpool_core::surfnet::{
    locker::SurfnetSvmLocker,
    svm::{SurfnetSvm, SurfnetSvmConfig},
};
use surfpool_sdk::cheatcodes::builders::{CheatcodeBuilder, SetAccount};
use surfpool_types::{
    parse_feature_pubkey, BlockProductionMode, RpcConfig, SimnetCommand, SimnetConfig, SimnetEvent, SurfpoolConfig,
    SvmFeatureConfig,
};
use tokio::runtime::Runtime;

use crate::utils::cu_utils::CuTracker;

pub use rewards_program_client::REWARDS_PROGRAM_ID as PROGRAM_ID;

const MIN_LAMPORTS: u64 = 500_000_000;
const CU_TRACKING_ENV_VAR: &str = "CU_TRACKING";
const PROGRAM_SO_RELATIVE_PATH: &[&str] = &["..", "..", "target", "deploy", "rewards_program.so"];

#[derive(Debug)]
struct TransactionFailure {
    err: TransactionError,
    logs: Vec<String>,
}

pub struct TestContext {
    surfnet: TestSurfnet,
    pub payer: Keypair,
    pub authority: Keypair,
    pub cu_tracker: Option<CuTracker>,
    current_timestamp: i64,
    current_slot: u64,
}

impl TestContext {
    pub fn new() -> Self {
        let payer = Keypair::new();
        let authority = Keypair::new();
        let current_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

        let surfnet = TestSurfnet::start().expect("Surfnet should start");

        surfnet.fund_sol(&payer.pubkey(), 100 * LAMPORTS_PER_SOL).expect("Payer should be funded");
        surfnet.fund_sol(&authority.pubkey(), 100 * LAMPORTS_PER_SOL).expect("Authority should be funded");
        surfnet.deploy_program(PROGRAM_ID, program_so_path()).expect("Rewards program should deploy to Surfnet");

        let cu_tracker = if std::env::var(CU_TRACKING_ENV_VAR).is_ok() { CuTracker::new() } else { None };

        Self { surfnet, payer, authority, cu_tracker, current_timestamp, current_slot: 1 }
    }

    pub fn airdrop_if_required(&mut self, pubkey: &Pubkey, lamports: u64) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(account) = self.get_account(pubkey) {
            if account.lamports < MIN_LAMPORTS {
                return match self.surfnet.fund_sol(pubkey, lamports) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!("Airdrop failed: {:?}", e).into()),
                };
            }
        } else {
            return match self.surfnet.fund_sol(pubkey, lamports) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Airdrop failed: {:?}", e).into()),
            };
        }

        Ok(())
    }

    pub fn send_transaction(
        &mut self,
        instruction: Instruction,
        signers: &[&Keypair],
    ) -> Result<u64, Box<dyn std::error::Error>> {
        self.send_transaction_inner(instruction, signers).map_err(|failure| {
            let logs = failure.logs.join("\n");
            format!("Transaction failed: {:?}\n{}", failure.err, logs).into()
        })
    }

    pub fn send_transaction_expect_error(
        &mut self,
        instruction: Instruction,
        signers: &[&Keypair],
    ) -> TransactionError {
        self.send_transaction_inner(instruction, signers).expect_err("Transaction should fail").err
    }

    fn send_transaction_inner(
        &mut self,
        instruction: Instruction,
        signers: &[&Keypair],
    ) -> Result<u64, TransactionFailure> {
        let mut all_signers = vec![&self.payer as &dyn Signer];
        all_signers.extend(signers.iter().map(|k| *k as &dyn Signer));

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer.pubkey()),
            &all_signers,
            self.surfnet.rpc_client().get_latest_blockhash().expect("latest blockhash should be available"),
        );

        let simulation = self
            .surfnet
            .rpc_client()
            .simulate_transaction_with_config(
                &transaction,
                RpcSimulateTransactionConfig {
                    encoding: Some(UiTransactionEncoding::Base64),
                    ..RpcSimulateTransactionConfig::default()
                },
            )
            .expect("transaction simulation should complete");

        if let Some(err) = simulation.value.err {
            return Err(TransactionFailure { err: err.into(), logs: simulation.value.logs.unwrap_or_default() });
        }

        self.surfnet
            .rpc_client()
            .send_transaction_with_config(
                &transaction,
                RpcSendTransactionConfig {
                    skip_preflight: true,
                    encoding: Some(UiTransactionEncoding::Base64),
                    ..RpcSendTransactionConfig::default()
                },
            )
            .expect("transaction should send after successful simulation");

        if let Ok(epoch_info) = self.surfnet.rpc_client().get_epoch_info() {
            self.current_slot = epoch_info.absolute_slot;
        }

        Ok(simulation.value.units_consumed.unwrap_or_default())
    }

    pub fn get_account(&self, pubkey: &Pubkey) -> Option<Account> {
        self.surfnet.rpc_client().get_account(pubkey).ok()
    }

    pub fn set_account(&mut self, pubkey: Pubkey, account: Account) {
        self.surfnet.set_account(pubkey, account).unwrap();
    }

    pub fn minimum_balance_for_rent_exemption(&self, data_len: usize) -> u64 {
        self.surfnet
            .rpc_client()
            .get_minimum_balance_for_rent_exemption(data_len)
            .expect("rent exemption should be available")
    }

    pub fn create_funded_keypair(&mut self) -> Keypair {
        let kp = Keypair::new();
        self.surfnet.fund_sol(&kp.pubkey(), MIN_LAMPORTS).unwrap();
        kp
    }

    pub fn warp_to_timestamp(&mut self, unix_timestamp: i64) {
        let timestamp_ms = u64::try_from(unix_timestamp).expect("timestamp should be positive") * 1000;
        let epoch_info = self.surfnet.time_travel_to_timestamp(timestamp_ms).unwrap();
        self.current_timestamp = unix_timestamp;
        self.current_slot = epoch_info.absolute_slot;
    }

    pub fn get_current_timestamp(&self) -> i64 {
        self.current_timestamp
    }

    pub fn warp_to_slot(&mut self, slot: u64) {
        let epoch_info = self.surfnet.time_travel_to_slot(slot).unwrap();
        self.current_slot = epoch_info.absolute_slot;
    }

    pub fn advance_slot(&mut self) {
        let current_slot = self.surfnet.current_absolute_slot().unwrap_or(self.current_slot);
        let next_slot = current_slot.max(self.current_slot).saturating_add(1);
        self.warp_to_slot(next_slot);
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

fn program_so_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for segment in PROGRAM_SO_RELATIVE_PATH {
        path.push(segment);
    }
    path
}

struct TestSurfnet {
    rpc_url: String,
    simnet_commands_tx: Sender<SimnetCommand>,
    _simnet_events_rx: Receiver<SimnetEvent>,
    _runloop_handle: std::thread::JoinHandle<()>,
}

impl TestSurfnet {
    fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let bind_port = get_free_port()?;
        let ws_port = get_free_port()?;
        let bind_host = "127.0.0.1".to_string();

        let surfpool_config = SurfpoolConfig {
            simnets: vec![SimnetConfig {
                offline_mode: true,
                remote_rpc_url: None,
                slot_time: 1,
                block_production_mode: BlockProductionMode::Transaction,
                skip_blockhash_check: false,
                ..Default::default()
            }],
            rpc: RpcConfig { bind_host: bind_host.clone(), bind_port, ws_port, ..Default::default() },
            ..Default::default()
        };

        let svm_config = SurfnetSvmConfig {
            surfnet_id: surfpool_config.simnets[0].surfnet_id.clone(),
            slot_time: surfpool_config.simnets[0].slot_time,
            instruction_profiling_enabled: surfpool_config.simnets[0].instruction_profiling_enabled,
            max_profiles: surfpool_config.simnets[0].max_profiles,
            log_bytes_limit: surfpool_config.simnets[0].log_bytes_limit,
            feature_config: all_enabled_feature_config(),
            skip_blockhash_check: surfpool_config.simnets[0].skip_blockhash_check,
        };

        let (surfnet_svm, simnet_events_rx, geyser_events_rx) = SurfnetSvm::new(svm_config)?;
        let (simnet_commands_tx, simnet_commands_rx) = crossbeam_channel::unbounded();

        let svm_locker = SurfnetSvmLocker::new(surfnet_svm);
        let svm_locker_clone = svm_locker.clone();
        let simnet_commands_tx_clone = simnet_commands_tx.clone();

        let runloop_handle = std::thread::Builder::new().name("surfnet-integration-tests".into()).spawn(move || {
            let runtime = Runtime::new().expect("Tokio runtime should start");
            let result = runtime.block_on(surfpool_core::runloops::start_local_surfnet_runloop(
                svm_locker_clone,
                surfpool_config,
                simnet_commands_tx_clone,
                simnet_commands_rx,
                geyser_events_rx,
            ));
            if let Err(err) = result {
                eprintln!("Surfnet exited with error: {err}");
            }
        })?;

        wait_for_ready(&simnet_events_rx)?;

        Ok(Self {
            rpc_url: format!("http://{bind_host}:{bind_port}"),
            simnet_commands_tx,
            _simnet_events_rx: simnet_events_rx,
            _runloop_handle: runloop_handle,
        })
    }

    fn rpc_client(&self) -> RpcClient {
        RpcClient::new(self.rpc_url.clone())
    }

    fn current_absolute_slot(&self) -> Result<u64, Box<dyn std::error::Error>> {
        self.rpc_client().get_epoch_info().map(|epoch_info| epoch_info.absolute_slot).map_err(|err| err.into())
    }

    fn fund_sol(&self, address: &Pubkey, lamports: u64) -> Result<(), Box<dyn std::error::Error>> {
        self.call_cheatcode("surfnet_setAccount", serde_json::json!([address.to_string(), { "lamports": lamports }]))
    }

    fn set_account(&self, pubkey: Pubkey, account: Account) -> Result<(), Box<dyn std::error::Error>> {
        self.call_cheatcode(
            SetAccount::METHOD,
            SetAccount::new(pubkey)
                .lamports(account.lamports)
                .data(account.data)
                .owner(account.owner)
                .executable(account.executable)
                .rent_epoch(account.rent_epoch)
                .build(),
        )
    }

    fn deploy_program(&self, program_id: Pubkey, so_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        const PROGRAM_CHUNK_BYTES: usize = 15 * 1024 * 1024;

        let program_bytes =
            std::fs::read(&so_path).map_err(|err| format!("failed to read {}: {err}", so_path.display()))?;

        for (index, chunk) in program_bytes.chunks(PROGRAM_CHUNK_BYTES).enumerate() {
            let offset = index * PROGRAM_CHUNK_BYTES;
            self.call_cheatcode(
                "surfnet_writeProgram",
                serde_json::json!([program_id.to_string(), hex::encode(chunk), offset]),
            )?;
        }

        Ok(())
    }

    fn time_travel_to_timestamp(&self, timestamp: u64) -> Result<EpochInfo, Box<dyn std::error::Error>> {
        self.time_travel(serde_json::json!([{ "absoluteTimestamp": timestamp }]))
    }

    fn time_travel_to_slot(&self, slot: u64) -> Result<EpochInfo, Box<dyn std::error::Error>> {
        self.time_travel(serde_json::json!([{ "absoluteSlot": slot }]))
    }

    fn time_travel(&self, params: serde_json::Value) -> Result<EpochInfo, Box<dyn std::error::Error>> {
        self.rpc_client()
            .send::<EpochInfo>(RpcRequest::Custom { method: "surfnet_timeTravel" }, params)
            .map_err(|err| format!("surfnet_timeTravel: {err}").into())
    }

    fn call_cheatcode(
        &self,
        method: &'static str,
        params: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.rpc_client()
            .send::<serde_json::Value>(RpcRequest::Custom { method }, params)
            .map(|_| ())
            .map_err(|err| format!("{method}: {err}").into())
    }
}

impl Drop for TestSurfnet {
    fn drop(&mut self) {
        let _ = self.simnet_commands_tx.send(SimnetCommand::Terminate(None));
    }
}

fn get_free_port() -> Result<u16, Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

fn wait_for_ready(events_rx: &Receiver<SimnetEvent>) -> Result<(), Box<dyn std::error::Error>> {
    let deadline = Instant::now() + Duration::from_secs(10);

    loop {
        if Instant::now() >= deadline {
            return Err("Surfnet did not become ready before timeout".into());
        }

        match events_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(SimnetEvent::Ready(_)) => return Ok(()),
            Ok(SimnetEvent::Aborted(err)) => return Err(err.into()),
            Ok(SimnetEvent::Shutdown) => return Err("Surfnet shut down during startup".into()),
            Ok(_) => continue,
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                sleep(Duration::from_millis(10));
            }
            Err(err) => return Err(format!("Surfnet events channel closed: {err}").into()),
        }
    }
}

fn all_enabled_feature_config() -> SvmFeatureConfig {
    // SurfnetSvm only applies non-default feature configs; forcing one known enable
    // preserves the all-enabled feature set used by the previous test harness.
    let feature = parse_feature_pubkey("enable_loader_v4").expect("feature name should resolve");
    SvmFeatureConfig::new().enable(feature)
}
