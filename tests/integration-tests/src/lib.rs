pub mod fixtures;
pub mod utils;

#[cfg(test)]
mod test_add_direct_recipient;
#[cfg(test)]
mod test_claim_continuous;
#[cfg(test)]
mod test_claim_direct;
#[cfg(test)]
mod test_claim_merkle;
#[cfg(test)]
mod test_cliff_vesting;
#[cfg(test)]
mod test_close_direct_distribution;
#[cfg(test)]
mod test_close_direct_recipient;
#[cfg(test)]
mod test_close_merkle_claim;
#[cfg(test)]
mod test_close_merkle_distribution;
#[cfg(test)]
mod test_close_reward_pool;
#[cfg(test)]
mod test_continuous_lifecycle;
#[cfg(test)]
mod test_create_direct_distribution;
#[cfg(test)]
mod test_create_merkle_distribution;
#[cfg(test)]
mod test_distribute_reward;
#[cfg(test)]
mod test_opt_in;
#[cfg(test)]
mod test_opt_out;
#[cfg(test)]
mod test_revoke_direct_recipient;
#[cfg(test)]
mod test_revoke_merkle_claim;
#[cfg(test)]
mod test_revoke_user;
#[cfg(test)]
mod test_set_balance;
#[cfg(test)]
mod test_sync_balance;
