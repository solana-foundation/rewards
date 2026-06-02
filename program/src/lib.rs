//! # Rewards Program
//!
//! A Solana program for managing token rewards distributions with vesting
//! schedules and claimable allocations.
//!
//! ## Features
//! - Direct distributions with explicit recipient lists
//! - Merkle distributions with proof-based claims
//! - Configurable vesting schedules for both distribution types
//! - Claimable rewards calculations
//! - Token-2022 extension blocking
//!
//! ## Architecture
//! Built with Pinocchio (no_std). Clients auto-generated via Codama.

#![no_std]

extern crate alloc;

use pinocchio::address::declare_id;

pub mod errors;
pub mod traits;
pub mod utils;

pub mod events;
pub mod instructions;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

declare_id!("REWArDioXgQJ2fZKkfu9LCLjQfRwYWVVfsvcsR5hoXi");

#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "Rewards Program",
    project_url: "https://github.com/solana-foundation/rewards",
    contacts: "link:https://github.com/solana-foundation/rewards/security/advisories/new",
    policy: "https://github.com/solana-foundation/rewards/security/policy",
    source_code: "https://github.com/solana-foundation/rewards"
}
