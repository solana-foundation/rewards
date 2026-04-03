use crate::define_instruction;

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
