use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventSerialize};

#[derive(CodamaType)]
pub struct BalanceSyncedEvent {
    pub reward_pool: Address,
    pub user: Address,
    pub old_balance: u64,
    pub new_balance: u64,
}

impl EventDiscriminator for BalanceSyncedEvent {
    const DISCRIMINATOR: [u8; 8] = [0x5d, 0xd3, 0xbd, 0xe4, 0xb7, 0x96, 0xf9, 0xfb];
}

impl EventSerialize for BalanceSyncedEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.reward_pool.as_ref());
        data.extend_from_slice(self.user.as_ref());
        data.extend_from_slice(&self.old_balance.to_le_bytes());
        data.extend_from_slice(&self.new_balance.to_le_bytes());
        data
    }
}

impl BalanceSyncedEvent {
    pub const DATA_LEN: usize = 32 + 32 + 8 + 8; // reward_pool + user + old_balance + new_balance

    #[inline(always)]
    pub fn new(reward_pool: Address, user: Address, old_balance: u64, new_balance: u64) -> Self {
        Self { reward_pool, user, old_balance, new_balance }
    }
}
