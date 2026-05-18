import type { Address, Rpc, SolanaRpcApi } from '@solana/kit';
import {
    type DirectDistribution,
    type DirectRecipient,
    type MerkleDistribution,
    type VestingSchedule,
    type VestingScheduleArgs,
} from '@solana/rewards';

import type { VestingScheduleState } from '@/lib/vesting-schedule';

export type RewardsRpc = Rpc<SolanaRpcApi>;

export type RewardKind = 'direct' | 'merkle';

export interface RewardCampaign {
    address: Address;
    authority: Address;
    clawbackTs: bigint;
    claimedAmount: bigint;
    kind: RewardKind;
    mint: Address;
    revocable: boolean;
    seed: Address;
    totalAmount: bigint;
}

export interface DirectRewardAllocation {
    address: Address;
    claimedAmount: bigint;
    distribution: Address;
    mint: Address | null;
    payer: Address;
    recipient: Address;
    remainingAmount: bigint;
    schedule: VestingSchedule;
    totalAmount: bigint;
}

export interface ProofDropClaimInput {
    id: string;
    distribution: Address;
    proof: readonly (readonly number[])[];
    proofText: string;
    schedule: VestingScheduleArgs;
    scheduleState: VestingScheduleState;
    totalAmount: bigint;
    updatedAt: number;
}

export interface ProofDropRewardAllocation extends ProofDropClaimInput {
    claimAccount: Address;
    claimedAmount: bigint;
    dropExists: boolean;
    mint: Address | null;
    remainingAmount: bigint;
}

export type ClaimableReward =
    | {
          allocation: DirectRewardAllocation;
          id: string;
          kind: 'recipient';
      }
    | {
          allocation: ProofDropRewardAllocation;
          id: string;
          kind: 'proof';
      };

export function directDistributionToCampaign(address: Address, data: DirectDistribution): RewardCampaign {
    return {
        address,
        authority: data.authority,
        clawbackTs: data.clawbackTs,
        claimedAmount: data.totalClaimed,
        kind: 'direct',
        mint: data.mint,
        revocable: data.revocable === 1,
        seed: data.seed,
        totalAmount: data.totalAllocated,
    };
}

export function merkleDistributionToCampaign(address: Address, data: MerkleDistribution): RewardCampaign {
    return {
        address,
        authority: data.authority,
        clawbackTs: data.clawbackTs,
        claimedAmount: data.totalClaimed,
        kind: 'merkle',
        mint: data.mint,
        revocable: data.revocable === 1,
        seed: data.seed,
        totalAmount: data.totalAmount,
    };
}

export function directRecipientToAllocation(
    address: Address,
    data: DirectRecipient,
    distributionMint: Address | null,
): DirectRewardAllocation {
    const remainingAmount = data.totalAmount > data.claimedAmount ? data.totalAmount - data.claimedAmount : 0n;

    return {
        address,
        claimedAmount: data.claimedAmount,
        distribution: data.distribution,
        mint: distributionMint,
        payer: data.payer,
        recipient: data.recipient,
        remainingAmount,
        schedule: data.schedule,
        totalAmount: data.totalAmount,
    };
}

export function formatTokenAmount(value: bigint): string {
    return new Intl.NumberFormat('en-US').format(value);
}

export function formatAddress(value: string, size = 4): string {
    if (value.length <= size * 2 + 2) return value;
    return `${value.slice(0, size)}..${value.slice(-size)}`;
}

export function rewardStatus(campaign: RewardCampaign): 'clawback-ready' | 'complete' | 'open' {
    if (campaign.totalAmount > 0n && campaign.claimedAmount >= campaign.totalAmount) return 'complete';
    if (campaign.clawbackTs > 0n && BigInt(Math.floor(Date.now() / 1000)) >= campaign.clawbackTs) {
        return 'clawback-ready';
    }
    return 'open';
}

export function vestingLabel(schedule: VestingSchedule | VestingScheduleArgs): string {
    if (schedule.__kind === 'CliffLinear') return 'Cliff linear';
    return schedule.__kind;
}
