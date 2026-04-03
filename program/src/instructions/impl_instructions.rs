use crate::define_instruction;

use super::continuous::{
    claim::{ClaimContinuousAccounts, ClaimContinuousData},
    claim_merkle::{ClaimContinuousMerkleAccounts, ClaimContinuousMerkleData},
    close_pool::{CloseContinuousPoolAccounts, CloseContinuousPoolData},
    create_pool::{CreateContinuousPoolAccounts, CreateContinuousPoolData},
    distribute_reward::{DistributeContinuousRewardAccounts, DistributeContinuousRewardData},
    opt_in::{ContinuousOptInAccounts, ContinuousOptInData},
    opt_out::{ContinuousOptOutAccounts, ContinuousOptOutData},
    revoke_user::{RevokeContinuousUserAccounts, RevokeContinuousUserData},
    set_balance::{SetContinuousBalanceAccounts, SetContinuousBalanceData},
    set_merkle_root::{SetContinuousMerkleRootAccounts, SetContinuousMerkleRootData},
    sync_balance::{SyncContinuousBalanceAccounts, SyncContinuousBalanceData},
};
use super::direct::{
    add_recipient::{AddDirectRecipientAccounts, AddDirectRecipientData},
    claim::{ClaimDirectAccounts, ClaimDirectData},
    close_distribution::{CloseDirectDistributionAccounts, CloseDirectDistributionData},
    close_recipient::{CloseDirectRecipientAccounts, CloseDirectRecipientData},
    create_distribution::{CreateDirectDistributionAccounts, CreateDirectDistributionData},
    revoke_recipient::{RevokeDirectRecipientAccounts, RevokeDirectRecipientData},
};
use super::merkle::{
    claim::{ClaimMerkleAccounts, ClaimMerkleData},
    close_claim::{CloseMerkleClaimAccounts, CloseMerkleClaimData},
    close_distribution::{CloseMerkleDistributionAccounts, CloseMerkleDistributionData},
    create_distribution::{CreateMerkleDistributionAccounts, CreateMerkleDistributionData},
    revoke_claim::{RevokeMerkleClaimAccounts, RevokeMerkleClaimData},
};
use super::points::{
    close_points_account::{ClosePointsAccountAccounts, ClosePointsAccountData},
    close_points_config::{ClosePointsConfigAccounts, ClosePointsConfigData},
    init_points::{InitPointsAccounts, InitPointsData},
    issue_points::{IssuePointsAccounts, IssuePointsData},
    revoke_points::{RevokePointsAccounts, RevokePointsData},
    transfer_points::{TransferPointsAccounts, TransferPointsData},
    use_points::{UsePointsAccounts, UsePointsData},
};

// Direct Distribution
define_instruction!(AddDirectRecipient, AddDirectRecipientAccounts, AddDirectRecipientData);
define_instruction!(ClaimDirect, ClaimDirectAccounts, ClaimDirectData);
define_instruction!(CloseDirectDistribution, CloseDirectDistributionAccounts, CloseDirectDistributionData);
define_instruction!(CloseDirectRecipient, CloseDirectRecipientAccounts, CloseDirectRecipientData);
define_instruction!(CreateDirectDistribution, CreateDirectDistributionAccounts, CreateDirectDistributionData);
define_instruction!(RevokeDirectRecipient, RevokeDirectRecipientAccounts, RevokeDirectRecipientData);

// Merkle Distribution
define_instruction!(ClaimMerkle, ClaimMerkleAccounts, ClaimMerkleData);
define_instruction!(CloseMerkleClaim, CloseMerkleClaimAccounts, CloseMerkleClaimData);
define_instruction!(CloseMerkleDistribution, CloseMerkleDistributionAccounts, CloseMerkleDistributionData);
define_instruction!(CreateMerkleDistribution, CreateMerkleDistributionAccounts, CreateMerkleDistributionData);
define_instruction!(RevokeMerkleClaim, RevokeMerkleClaimAccounts, RevokeMerkleClaimData);

// Continuous Distribution
define_instruction!(CreateContinuousPool, CreateContinuousPoolAccounts, CreateContinuousPoolData);
define_instruction!(ContinuousOptIn, ContinuousOptInAccounts, ContinuousOptInData);
define_instruction!(ContinuousOptOut, ContinuousOptOutAccounts, ContinuousOptOutData);
define_instruction!(DistributeContinuousReward, DistributeContinuousRewardAccounts, DistributeContinuousRewardData);
define_instruction!(ClaimContinuous, ClaimContinuousAccounts, ClaimContinuousData);
define_instruction!(SetContinuousMerkleRoot, SetContinuousMerkleRootAccounts, SetContinuousMerkleRootData);
define_instruction!(ClaimContinuousMerkle, ClaimContinuousMerkleAccounts, ClaimContinuousMerkleData);
define_instruction!(SyncContinuousBalance, SyncContinuousBalanceAccounts, SyncContinuousBalanceData);
define_instruction!(SetContinuousBalance, SetContinuousBalanceAccounts, SetContinuousBalanceData);
define_instruction!(CloseContinuousPool, CloseContinuousPoolAccounts, CloseContinuousPoolData);
define_instruction!(RevokeContinuousUser, RevokeContinuousUserAccounts, RevokeContinuousUserData);

// Points
define_instruction!(InitPoints, InitPointsAccounts, InitPointsData);
define_instruction!(IssuePoints, IssuePointsAccounts, IssuePointsData);
define_instruction!(UsePoints, UsePointsAccounts, UsePointsData);
define_instruction!(TransferPoints, TransferPointsAccounts, TransferPointsData);
define_instruction!(ClosePointsAccount, ClosePointsAccountAccounts, ClosePointsAccountData);
define_instruction!(ClosePointsConfig, ClosePointsConfigAccounts, ClosePointsConfigData);
define_instruction!(RevokePoints, RevokePointsAccounts, RevokePointsData);
