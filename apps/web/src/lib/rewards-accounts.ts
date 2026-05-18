import {
    decodeDirectDistribution,
    decodeDirectRecipient,
    decodeMerkleDistribution,
    fetchAllMaybeDirectDistribution,
    fetchAllMaybeMerkleClaim,
    fetchAllMaybeMerkleDistribution,
    type MerkleClaim,
    type MerkleDistribution,
} from '@solana/rewards';
import {
    address,
    getBase64Encoder,
    type Address,
    type Base58EncodedBytes,
    type Base64EncodedBytes,
    type EncodedAccount,
    type Lamports,
} from '@solana/kit';

import { deriveMerkleClaimPda } from '@/lib/pdas';
import { getProgramAddress } from '@/lib/program';
import {
    directDistributionToCampaign,
    directRecipientToAllocation,
    merkleDistributionToCampaign,
    type ClaimableReward,
    type DirectRewardAllocation,
    type ProofDropClaimInput,
    type ProofDropRewardAllocation,
    type RewardCampaign,
    type RewardsRpc,
} from '@/lib/rewards-model';

const DIRECT_DISTRIBUTION_DISCRIMINATOR = 0;
const DIRECT_RECIPIENT_DISCRIMINATOR = 1;
const MERKLE_DISTRIBUTION_DISCRIMINATOR = 2;

const DISCRIMINATOR_OFFSET = 0n;
const DISTRIBUTION_AUTHORITY_OFFSET = 10n;
const DIRECT_RECIPIENT_RECIPIENT_OFFSET = 35n;
const DIRECT_DISTRIBUTION_SIZE = 130n;
const MERKLE_DISTRIBUTION_SIZE = 162n;
const DISCRIMINATOR_BYTES = ['AA==', 'AQ==', 'Ag=='] as const;

const base64Encoder = getBase64Encoder();

interface Base64ProgramAccount {
    account: {
        data: readonly [Base64EncodedBytes, 'base64'];
        executable: boolean;
        lamports: Lamports;
        owner: Address;
        space: bigint;
    };
    pubkey: Address;
}

function discriminatorBytes(discriminator: number): Base64EncodedBytes {
    return DISCRIMINATOR_BYTES[discriminator] as Base64EncodedBytes;
}

function addressBytes(address: Address): Base58EncodedBytes {
    return address as unknown as Base58EncodedBytes;
}

function toEncodedAccount(account: Base64ProgramAccount): EncodedAccount {
    const [base64Data] = account.account.data;
    return {
        address: account.pubkey,
        data: base64Encoder.encode(base64Data),
        executable: account.account.executable,
        lamports: account.account.lamports,
        programAddress: account.account.owner,
        space: account.account.space,
    };
}

export async function fetchCreatedRewardCampaigns(rpc: RewardsRpc, authority: Address): Promise<RewardCampaign[]> {
    const programAddress = getProgramAddress();
    const [directAccounts, merkleAccounts] = await Promise.all([
        rpc
            .getProgramAccounts(programAddress, {
                encoding: 'base64',
                filters: [
                    { dataSize: DIRECT_DISTRIBUTION_SIZE },
                    {
                        memcmp: {
                            bytes: discriminatorBytes(DIRECT_DISTRIBUTION_DISCRIMINATOR),
                            encoding: 'base64',
                            offset: DISCRIMINATOR_OFFSET,
                        },
                    },
                    {
                        memcmp: {
                            bytes: addressBytes(authority),
                            encoding: 'base58',
                            offset: DISTRIBUTION_AUTHORITY_OFFSET,
                        },
                    },
                ],
            })
            .send(),
        rpc
            .getProgramAccounts(programAddress, {
                encoding: 'base64',
                filters: [
                    { dataSize: MERKLE_DISTRIBUTION_SIZE },
                    {
                        memcmp: {
                            bytes: discriminatorBytes(MERKLE_DISTRIBUTION_DISCRIMINATOR),
                            encoding: 'base64',
                            offset: DISCRIMINATOR_OFFSET,
                        },
                    },
                    {
                        memcmp: {
                            bytes: addressBytes(authority),
                            encoding: 'base58',
                            offset: DISTRIBUTION_AUTHORITY_OFFSET,
                        },
                    },
                ],
            })
            .send(),
    ]);

    return [
        ...directAccounts.map(account => {
            const decoded = decodeDirectDistribution(toEncodedAccount(account));
            return directDistributionToCampaign(decoded.address, decoded.data);
        }),
        ...merkleAccounts.map(account => {
            const decoded = decodeMerkleDistribution(toEncodedAccount(account));
            return merkleDistributionToCampaign(decoded.address, decoded.data);
        }),
    ].sort((a, b) => a.kind.localeCompare(b.kind) || a.address.localeCompare(b.address));
}

export async function fetchDirectRewardAllocations(
    rpc: RewardsRpc,
    recipient: Address,
): Promise<DirectRewardAllocation[]> {
    const programAddress = getProgramAddress();
    const recipientAccounts = await rpc
        .getProgramAccounts(programAddress, {
            encoding: 'base64',
            filters: [
                {
                    memcmp: {
                        bytes: discriminatorBytes(DIRECT_RECIPIENT_DISCRIMINATOR),
                        encoding: 'base64',
                        offset: DISCRIMINATOR_OFFSET,
                    },
                },
                {
                    memcmp: {
                        bytes: addressBytes(recipient),
                        encoding: 'base58',
                        offset: DIRECT_RECIPIENT_RECIPIENT_OFFSET,
                    },
                },
            ],
        })
        .send();

    const decodedRecipients = recipientAccounts.map(account => decodeDirectRecipient(toEncodedAccount(account)));
    const distributions = [...new Set(decodedRecipients.map(account => account.data.distribution))];
    const distributionAccounts =
        distributions.length > 0 ? await fetchAllMaybeDirectDistribution(rpc, distributions).catch(() => []) : [];
    const mintByDistribution = new Map<Address, Address>(
        distributionAccounts
            .filter(account => account.exists)
            .map(account => [account.address, account.data.mint] as const),
    );

    return decodedRecipients
        .map(account =>
            directRecipientToAllocation(
                account.address,
                account.data,
                mintByDistribution.get(account.data.distribution) ?? null,
            ),
        )
        .sort((a, b) => {
            if (a.remainingAmount === b.remainingAmount) return a.address.localeCompare(b.address);
            return a.remainingAmount > b.remainingAmount ? -1 : 1;
        });
}

export async function fetchProofDropRewardAllocations(
    rpc: RewardsRpc,
    claimant: Address,
    proofDrops: readonly ProofDropClaimInput[],
): Promise<ProofDropRewardAllocation[]> {
    if (proofDrops.length === 0) return [];

    const programAddress = getProgramAddress();
    const distributionAddresses = [...new Set(proofDrops.map(drop => drop.distribution))];
    const distributionAccounts = await fetchAllMaybeMerkleDistribution(rpc, distributionAddresses).catch(() => []);
    const distributionByAddress = new Map<Address, MerkleDistribution>();
    for (const account of distributionAccounts) {
        if (account.exists) distributionByAddress.set(account.address, account.data);
    }

    const claimAddresses = proofDrops.map(drop => deriveMerkleClaimPda(drop.distribution, claimant, programAddress)[0]);
    const claimAccounts =
        claimAddresses.length > 0
            ? await fetchAllMaybeMerkleClaim(
                  rpc,
                  claimAddresses.map(claim => address(claim)),
              )
            : [];
    const claimedByAddress = new Map<Address, MerkleClaim['claimedAmount']>();
    for (const account of claimAccounts) {
        if (account.exists) claimedByAddress.set(account.address, account.data.claimedAmount);
    }

    return proofDrops.map(drop => {
        const claimAccount = address(deriveMerkleClaimPda(drop.distribution, claimant, programAddress)[0]);
        const distribution = distributionByAddress.get(drop.distribution);
        const claimedAmount = claimedByAddress.get(claimAccount) ?? 0n;
        const remainingAmount = drop.totalAmount > claimedAmount ? drop.totalAmount - claimedAmount : 0n;

        return {
            ...drop,
            claimAccount,
            claimedAmount,
            dropExists: Boolean(distribution),
            mint: distribution?.mint ?? null,
            remainingAmount,
        };
    });
}

export async function fetchClaimableRewards(
    rpc: RewardsRpc,
    claimant: Address,
    proofDrops: readonly ProofDropClaimInput[],
): Promise<ClaimableReward[]> {
    const [recipientAllocations, proofAllocations] = await Promise.all([
        fetchDirectRewardAllocations(rpc, claimant),
        fetchProofDropRewardAllocations(rpc, claimant, proofDrops),
    ]);

    return [
        ...recipientAllocations.map(allocation => ({
            allocation,
            id: `recipient:${allocation.address}`,
            kind: 'recipient' as const,
        })),
        ...proofAllocations.map(allocation => ({
            allocation,
            id: `proof:${allocation.id}`,
            kind: 'proof' as const,
        })),
    ].sort((a, b) => {
        const aRemaining = a.allocation.remainingAmount;
        const bRemaining = b.allocation.remainingAmount;
        if (aRemaining === bRemaining) return a.id.localeCompare(b.id);
        return aRemaining > bRemaining ? -1 : 1;
    });
}
