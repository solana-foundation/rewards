import { useWallet } from '@solana/connector/react';
import { address } from '@solana/kit';
import { useCallback, useEffect, useMemo, useState } from 'react';

import type { ProofDropClaimInput } from '@/lib/rewards-model';
import { parseBigIntValue, parseMerkleProof } from '@/lib/validation';
import { buildVestingSchedule, isVestingScheduleState, type VestingScheduleState } from '@/lib/vesting-schedule';

const STORAGE_KEY = 'rewards-ui-proof-drop-claims-v1';
const MAX_PROOF_DROPS = 50;

export interface StoredProofDropClaim {
    claimant: string;
    createdAt: number;
    distribution: string;
    id: string;
    proof: string;
    schedule: VestingScheduleState;
    totalAmount: string;
    updatedAt: number;
}

export interface ProofDropClaimDraft {
    distribution: string;
    proof: string;
    schedule: VestingScheduleState;
    totalAmount: string;
}

function normalize(value: string) {
    return value.trim();
}

function isStoredProofDropClaim(value: unknown): value is StoredProofDropClaim {
    if (!value || typeof value !== 'object') return false;
    const candidate = value as Record<string, unknown>;
    return (
        typeof candidate.claimant === 'string' &&
        typeof candidate.createdAt === 'number' &&
        typeof candidate.distribution === 'string' &&
        typeof candidate.id === 'string' &&
        typeof candidate.proof === 'string' &&
        isVestingScheduleState(candidate.schedule) &&
        typeof candidate.totalAmount === 'string' &&
        typeof candidate.updatedAt === 'number'
    );
}

function readProofDrops(): StoredProofDropClaim[] {
    try {
        const raw = window.localStorage.getItem(STORAGE_KEY);
        if (!raw) return [];
        const parsed: unknown = JSON.parse(raw);
        if (!Array.isArray(parsed)) return [];
        return parsed.filter(isStoredProofDropClaim).slice(0, MAX_PROOF_DROPS);
    } catch {
        return [];
    }
}

function claimId(claimant: string, distribution: string) {
    return `${claimant}:${distribution}`;
}

export function toProofDropClaimInput(claim: StoredProofDropClaim): ProofDropClaimInput | null {
    const proofResult = parseMerkleProof(claim.proof);
    if (!proofResult.ok) return null;

    const scheduleResult = buildVestingSchedule(claim.schedule);
    if (!scheduleResult.ok) return null;

    try {
        return {
            distribution: address(claim.distribution),
            id: claim.id,
            proof: proofResult.value,
            proofText: claim.proof,
            schedule: scheduleResult.value,
            scheduleState: claim.schedule,
            totalAmount: parseBigIntValue(claim.totalAmount),
            updatedAt: claim.updatedAt,
        };
    } catch {
        return null;
    }
}

export function useProofDropClaims() {
    const { account } = useWallet();
    const [claims, setClaims] = useState<StoredProofDropClaim[]>([]);
    const [hydrated, setHydrated] = useState(false);

    useEffect(() => {
        setClaims(readProofDrops());
        setHydrated(true);
    }, []);

    useEffect(() => {
        if (!hydrated) return;
        window.localStorage.setItem(STORAGE_KEY, JSON.stringify(claims.slice(0, MAX_PROOF_DROPS)));
    }, [claims, hydrated]);

    const walletClaims = useMemo(
        () => claims.filter(claim => claim.claimant === account).sort((a, b) => b.updatedAt - a.updatedAt),
        [account, claims],
    );

    const addProofDropClaim = useCallback(
        (draft: ProofDropClaimDraft) => {
            if (!account) throw new Error('Wallet not connected');

            const distribution = normalize(draft.distribution);
            const claimant = normalize(account);
            const now = Date.now();
            const id = claimId(claimant, distribution);
            const nextClaim: StoredProofDropClaim = {
                claimant,
                createdAt: now,
                distribution,
                id,
                proof: normalize(draft.proof),
                schedule: draft.schedule,
                totalAmount: normalize(draft.totalAmount),
                updatedAt: now,
            };

            setClaims(current =>
                [nextClaim, ...current.filter(claim => claim.id !== id || claim.claimant !== claimant)].slice(
                    0,
                    MAX_PROOF_DROPS,
                ),
            );
        },
        [account],
    );

    const removeProofDropClaim = useCallback((id: string) => {
        setClaims(current => current.filter(claim => claim.id !== id));
    }, []);

    return {
        addProofDropClaim,
        hydrated,
        proofDrops: walletClaims,
        removeProofDropClaim,
    };
}
