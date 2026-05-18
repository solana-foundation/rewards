import { useWallet } from '@solana/connector/react';
import { address } from '@solana/kit';
import { useQuery } from '@tanstack/react-query';

import { useClusterConfig } from '@/hooks/use-cluster-config';
import { useRpc } from '@/hooks/useRpc';
import { getProgramAddress } from '@/lib/program';
import { fetchClaimableRewards, fetchCreatedRewardCampaigns } from '@/lib/rewards-accounts';
import type { ProofDropClaimInput } from '@/lib/rewards-model';

export function useCreatedRewards() {
    const { account } = useWallet();
    const rpc = useRpc();
    const cluster = useClusterConfig();
    const programAddress = getProgramAddress();

    return useQuery({
        enabled: Boolean(account),
        queryFn: () => fetchCreatedRewardCampaigns(rpc, address(account!)),
        queryKey: ['rewards', 'created', account, cluster.id, programAddress],
    });
}

function proofDropClaimsKey(proofDrops: readonly ProofDropClaimInput[]) {
    return proofDrops.map(drop => `${drop.id}:${drop.updatedAt}`).join('|');
}

export function useClaimableRewards(proofDrops: readonly ProofDropClaimInput[] = []) {
    const { account } = useWallet();
    const rpc = useRpc();
    const cluster = useClusterConfig();
    const programAddress = getProgramAddress();

    return useQuery({
        enabled: Boolean(account),
        queryFn: () => fetchClaimableRewards(rpc, address(account!), proofDrops),
        queryKey: ['rewards', 'claimable', account, cluster.id, programAddress, proofDropClaimsKey(proofDrops)],
    });
}
