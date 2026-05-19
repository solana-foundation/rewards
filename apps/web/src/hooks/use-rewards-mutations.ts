import { useKitTransactionSigner } from '@solana/connector/react';
import { type Address, address, generateKeyPairSigner, type Instruction, type TransactionSigner } from '@solana/kit';
import {
    getAddDirectRecipientInstruction,
    getClaimDirectInstruction,
    getClaimMerkleInstruction,
    getCloseDirectDistributionInstruction,
    getCloseDirectRecipientInstruction,
    getCloseMerkleClaimInstruction,
    getCloseMerkleDistributionInstruction,
    getCreateDirectDistributionInstruction,
    getCreateMerkleDistributionInstruction,
    getRevokeDirectRecipientInstruction,
    getRevokeMerkleClaimInstruction,
    type RevokeMode,
    type VestingScheduleArgs,
} from '@solana/rewards';
import { useMutation, useQueryClient } from '@tanstack/react-query';

import { useWalletTransactionSignAndSend } from '@/components/solana/use-wallet-transaction-sign-and-send';
import { useTransactionToast } from '@/components/use-transaction-toast';
import { RECENT_VALUE_KEYS, type RecentTransactionValues } from '@/contexts/RecentTransactionsContext';
import { useRecentTransactions } from '@/contexts/RecentTransactionsContext';
import {
    deriveAta,
    deriveDirectDistributionPda,
    deriveDirectRecipientPda,
    deriveEventAuthority,
    deriveMerkleClaimPda,
    deriveMerkleDistributionPda,
    deriveRevocationPda,
    normalizeTokenProgram,
} from '@/lib/pdas';
import { getProgramAddress } from '@/lib/program';
import { formatTransactionError } from '@/lib/transactionErrors';
import { invalidateWithDelay } from '@/lib/utils';

interface RewardsMutationResult {
    readonly signature: string;
}

interface DistributionMutationResult extends RewardsMutationResult {
    readonly distribution: string;
    readonly mint: string;
    readonly seed: string;
}

export interface CreateDirectDistributionInput {
    readonly clawbackTs: bigint;
    readonly mint: string;
    readonly revocable: number;
    readonly tokenProgram: string;
}

export interface CreateMerkleDistributionInput extends CreateDirectDistributionInput {
    readonly amount: bigint;
    readonly merkleRoot: readonly number[];
    readonly totalAmount: bigint;
}

export interface AddDirectRecipientInput {
    readonly amount: bigint;
    readonly distribution: string;
    readonly mint: string;
    readonly recipient: string;
    readonly schedule: VestingScheduleArgs;
    readonly tokenProgram: string;
}

export interface CloseDistributionInput {
    readonly distribution: string;
    readonly mint: string;
    readonly tokenProgram: string;
}

export interface CloseDirectRecipientInput {
    readonly distribution: string;
    readonly originalPayer: string;
}

export interface CloseMerkleClaimInput {
    readonly distribution: string;
}

export interface ClaimDirectInput {
    readonly amount: bigint;
    readonly distribution: string;
    readonly mint: string;
    readonly tokenProgram: string;
}

export interface ClaimMerkleInput extends ClaimDirectInput {
    readonly proof: readonly (readonly number[])[];
    readonly schedule: VestingScheduleArgs;
    readonly totalAmount: bigint;
}

export interface RevokeDirectRecipientInput {
    readonly distribution: string;
    readonly mint: string;
    readonly originalPayer: string;
    readonly recipient: string;
    readonly revokeMode: RevokeMode;
    readonly tokenProgram: string;
}

export interface RevokeMerkleClaimInput {
    readonly claimant: string;
    readonly distribution: string;
    readonly mint: string;
    readonly proof: readonly (readonly number[])[];
    readonly revokeMode: RevokeMode;
    readonly schedule: VestingScheduleArgs;
    readonly tokenProgram: string;
    readonly totalAmount: bigint;
}

function asAddress(value: string): Address {
    return address(value.trim());
}

function getProgramConfig() {
    const programAddress = getProgramAddress();
    return {
        eventAuthority: deriveEventAuthority(programAddress),
        programAddress,
    } as const;
}

function createTransactionId(): string {
    return `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

function normalizeValues(values?: RecentTransactionValues): RecentTransactionValues | undefined {
    if (!values) return undefined;
    const normalized: RecentTransactionValues = {};
    for (const key of RECENT_VALUE_KEYS) {
        const value = values[key]?.trim();
        if (value) normalized[key] = value;
    }
    return Object.keys(normalized).length > 0 ? normalized : undefined;
}

export function useRewardsMutations() {
    const { signer } = useKitTransactionSigner();
    const signAndSend = useWalletTransactionSignAndSend();
    const { addRecentTransaction } = useRecentTransactions();
    const queryClient = useQueryClient();
    const transactionToast = useTransactionToast();

    function requireSigner(): TransactionSigner {
        if (!signer) throw new Error('Wallet not connected');
        return signer;
    }

    async function sendRewardTransaction(
        instructions: readonly Instruction[],
        txSigner: TransactionSigner,
        action: string,
        values?: RecentTransactionValues,
    ): Promise<string> {
        const normalizedValues = normalizeValues(values);
        const id = createTransactionId();

        try {
            const signature = await signAndSend(instructions, txSigner);
            addRecentTransaction({
                action,
                id,
                signature,
                status: 'success',
                timestamp: Date.now(),
                values: normalizedValues,
            });
            return signature;
        } catch (error) {
            const message = formatTransactionError(error);
            addRecentTransaction({
                action,
                error: message,
                id,
                signature: null,
                status: 'failed',
                timestamp: Date.now(),
                values: normalizedValues,
            });
            throw new Error(message, { cause: error });
        }
    }

    function onSuccess(result: RewardsMutationResult) {
        transactionToast.onSuccess(result.signature);
        invalidateWithDelay(queryClient, [['rewards']]);
    }

    function onError(error: unknown) {
        transactionToast.onError(error);
    }

    const createDirectDistribution = useMutation({
        mutationFn: async (input: CreateDirectDistributionInput): Promise<DistributionMutationResult> => {
            const txSigner = requireSigner();
            const { programAddress, eventAuthority } = getProgramConfig();
            const mint = input.mint.trim();
            const tokenProgram = normalizeTokenProgram(input.tokenProgram);
            const seedSigner = await generateKeyPairSigner();
            const [distribution, bump] = deriveDirectDistributionPda(
                mint,
                txSigner.address,
                seedSigner.address,
                programAddress,
            );
            const distributionVault = deriveAta(distribution, mint, tokenProgram);

            const instruction = getCreateDirectDistributionInstruction(
                {
                    authority: txSigner,
                    bump,
                    clawbackTs: input.clawbackTs,
                    distribution: asAddress(distribution),
                    distributionVault,
                    eventAuthority,
                    mint: asAddress(mint),
                    payer: txSigner,
                    revocable: input.revocable,
                    seeds: seedSigner,
                    tokenProgram,
                },
                { programAddress },
            );

            const signature = await sendRewardTransaction([instruction], txSigner, 'Create Direct Distribution', {
                distribution,
                mint,
            });
            return { distribution, mint, seed: seedSigner.address, signature };
        },
        onError,
        onSuccess,
    });

    const createMerkleDistribution = useMutation({
        mutationFn: async (input: CreateMerkleDistributionInput): Promise<DistributionMutationResult> => {
            const txSigner = requireSigner();
            const { programAddress, eventAuthority } = getProgramConfig();
            const mint = input.mint.trim();
            const tokenProgram = normalizeTokenProgram(input.tokenProgram);
            const seedSigner = await generateKeyPairSigner();
            const [distribution, bump] = deriveMerkleDistributionPda(
                mint,
                txSigner.address,
                seedSigner.address,
                programAddress,
            );
            const distributionVault = deriveAta(distribution, mint, tokenProgram);
            const authorityTokenAccount = deriveAta(txSigner.address, mint, tokenProgram);

            const instruction = getCreateMerkleDistributionInstruction(
                {
                    amount: input.amount,
                    authority: txSigner,
                    authorityTokenAccount,
                    bump,
                    clawbackTs: input.clawbackTs,
                    distribution: asAddress(distribution),
                    distributionVault,
                    eventAuthority,
                    merkleRoot: [...input.merkleRoot],
                    mint: asAddress(mint),
                    payer: txSigner,
                    revocable: input.revocable,
                    seeds: seedSigner,
                    tokenProgram,
                    totalAmount: input.totalAmount,
                },
                { programAddress },
            );

            const signature = await sendRewardTransaction([instruction], txSigner, 'Create Merkle Distribution', {
                amount: String(input.amount),
                distribution,
                mint,
                totalAmount: String(input.totalAmount),
            });
            return { distribution, mint, seed: seedSigner.address, signature };
        },
        onError,
        onSuccess,
    });

    const addDirectRecipient = useMutation({
        mutationFn: async (input: AddDirectRecipientInput): Promise<RewardsMutationResult> => {
            const txSigner = requireSigner();
            const { programAddress, eventAuthority } = getProgramConfig();
            const distribution = input.distribution.trim();
            const mint = input.mint.trim();
            const recipient = input.recipient.trim();
            const tokenProgram = normalizeTokenProgram(input.tokenProgram);
            const [recipientAccount, bump] = deriveDirectRecipientPda(distribution, recipient, programAddress);
            const distributionVault = deriveAta(distribution, mint, tokenProgram);
            const authorityTokenAccount = deriveAta(txSigner.address, mint, tokenProgram);

            const instruction = getAddDirectRecipientInstruction(
                {
                    amount: input.amount,
                    authority: txSigner,
                    authorityTokenAccount,
                    bump,
                    distribution: asAddress(distribution),
                    distributionVault,
                    eventAuthority,
                    mint: asAddress(mint),
                    payer: txSigner,
                    recipient: asAddress(recipient),
                    recipientAccount: asAddress(recipientAccount),
                    schedule: input.schedule,
                    tokenProgram,
                },
                { programAddress },
            );

            const signature = await sendRewardTransaction([instruction], txSigner, 'Add Direct Recipient', {
                amount: String(input.amount),
                distribution,
                mint,
                recipient,
            });
            return { signature };
        },
        onError,
        onSuccess,
    });

    const closeDirectDistribution = useMutation({
        mutationFn: async (input: CloseDistributionInput): Promise<RewardsMutationResult> => {
            const txSigner = requireSigner();
            const { programAddress, eventAuthority } = getProgramConfig();
            const distribution = input.distribution.trim();
            const mint = input.mint.trim();
            const tokenProgram = normalizeTokenProgram(input.tokenProgram);
            const distributionVault = deriveAta(distribution, mint, tokenProgram);
            const authorityTokenAccount = deriveAta(txSigner.address, mint, tokenProgram);

            const instruction = getCloseDirectDistributionInstruction(
                {
                    authority: txSigner,
                    authorityTokenAccount,
                    distribution: asAddress(distribution),
                    distributionVault,
                    eventAuthority,
                    mint: asAddress(mint),
                    tokenProgram,
                },
                { programAddress },
            );

            const signature = await sendRewardTransaction([instruction], txSigner, 'Close Direct Distribution', {
                distribution,
                mint,
            });
            return { signature };
        },
        onError,
        onSuccess,
    });

    const closeMerkleDistribution = useMutation({
        mutationFn: async (input: CloseDistributionInput): Promise<RewardsMutationResult> => {
            const txSigner = requireSigner();
            const { programAddress, eventAuthority } = getProgramConfig();
            const distribution = input.distribution.trim();
            const mint = input.mint.trim();
            const tokenProgram = normalizeTokenProgram(input.tokenProgram);
            const distributionVault = deriveAta(distribution, mint, tokenProgram);
            const authorityTokenAccount = deriveAta(txSigner.address, mint, tokenProgram);

            const instruction = getCloseMerkleDistributionInstruction(
                {
                    authority: txSigner,
                    authorityTokenAccount,
                    distribution: asAddress(distribution),
                    distributionVault,
                    eventAuthority,
                    mint: asAddress(mint),
                    tokenProgram,
                },
                { programAddress },
            );

            const signature = await sendRewardTransaction([instruction], txSigner, 'Close Merkle Distribution', {
                distribution,
                mint,
            });
            return { signature };
        },
        onError,
        onSuccess,
    });

    const closeDirectRecipient = useMutation({
        mutationFn: async (input: CloseDirectRecipientInput): Promise<RewardsMutationResult> => {
            const txSigner = requireSigner();
            const { programAddress, eventAuthority } = getProgramConfig();
            const distribution = input.distribution.trim();
            const originalPayer = input.originalPayer.trim();
            const [recipientAccount] = deriveDirectRecipientPda(distribution, txSigner.address, programAddress);

            const instruction = getCloseDirectRecipientInstruction(
                {
                    distribution: asAddress(distribution),
                    eventAuthority,
                    originalPayer: asAddress(originalPayer),
                    recipient: txSigner.address,
                    recipientAccount: asAddress(recipientAccount),
                },
                { programAddress },
            );

            const signature = await sendRewardTransaction([instruction], txSigner, 'Close Direct Recipient', {
                distribution,
            });
            return { signature };
        },
        onError,
        onSuccess,
    });

    const closeMerkleClaim = useMutation({
        mutationFn: async (input: CloseMerkleClaimInput): Promise<RewardsMutationResult> => {
            const txSigner = requireSigner();
            const { programAddress, eventAuthority } = getProgramConfig();
            const distribution = input.distribution.trim();
            const [claimAccount] = deriveMerkleClaimPda(distribution, txSigner.address, programAddress);

            const instruction = getCloseMerkleClaimInstruction(
                {
                    claimAccount: asAddress(claimAccount),
                    claimant: txSigner,
                    distribution: asAddress(distribution),
                    eventAuthority,
                },
                { programAddress },
            );

            const signature = await sendRewardTransaction([instruction], txSigner, 'Close Merkle Claim', {
                distribution,
            });
            return { signature };
        },
        onError,
        onSuccess,
    });

    const claimDirect = useMutation({
        mutationFn: async (input: ClaimDirectInput): Promise<RewardsMutationResult> => {
            const txSigner = requireSigner();
            const { programAddress, eventAuthority } = getProgramConfig();
            const distribution = input.distribution.trim();
            const mint = input.mint.trim();
            const tokenProgram = normalizeTokenProgram(input.tokenProgram);
            const [recipientAccount] = deriveDirectRecipientPda(distribution, txSigner.address, programAddress);
            const distributionVault = deriveAta(distribution, mint, tokenProgram);
            const recipientTokenAccount = deriveAta(txSigner.address, mint, tokenProgram);

            const instruction = getClaimDirectInstruction(
                {
                    amount: input.amount,
                    distribution: asAddress(distribution),
                    distributionVault,
                    eventAuthority,
                    mint: asAddress(mint),
                    recipient: txSigner,
                    recipientAccount: asAddress(recipientAccount),
                    recipientTokenAccount,
                    tokenProgram,
                },
                { programAddress },
            );

            const signature = await sendRewardTransaction([instruction], txSigner, 'Claim Direct', {
                amount: String(input.amount),
                distribution,
                mint,
            });
            return { signature };
        },
        onError,
        onSuccess,
    });

    const claimMerkle = useMutation({
        mutationFn: async (input: ClaimMerkleInput): Promise<RewardsMutationResult> => {
            const txSigner = requireSigner();
            const { programAddress, eventAuthority } = getProgramConfig();
            const distribution = input.distribution.trim();
            const mint = input.mint.trim();
            const tokenProgram = normalizeTokenProgram(input.tokenProgram);
            const [claimAccount, claimBump] = deriveMerkleClaimPda(distribution, txSigner.address, programAddress);
            const [revocationMarker] = deriveRevocationPda(distribution, txSigner.address, programAddress);
            const distributionVault = deriveAta(distribution, mint, tokenProgram);
            const claimantTokenAccount = deriveAta(txSigner.address, mint, tokenProgram);

            const instruction = getClaimMerkleInstruction(
                {
                    amount: input.amount,
                    claimAccount: asAddress(claimAccount),
                    claimBump,
                    claimant: txSigner,
                    claimantTokenAccount,
                    distribution: asAddress(distribution),
                    distributionVault,
                    eventAuthority,
                    mint: asAddress(mint),
                    payer: txSigner,
                    proof: input.proof.map(node => [...node]),
                    revocationMarker: asAddress(revocationMarker),
                    schedule: input.schedule,
                    tokenProgram,
                    totalAmount: input.totalAmount,
                },
                { programAddress },
            );

            const signature = await sendRewardTransaction([instruction], txSigner, 'Claim Merkle', {
                amount: String(input.amount),
                distribution,
                mint,
                totalAmount: String(input.totalAmount),
            });
            return { signature };
        },
        onError,
        onSuccess,
    });

    const revokeDirectRecipient = useMutation({
        mutationFn: async (input: RevokeDirectRecipientInput): Promise<RewardsMutationResult> => {
            const txSigner = requireSigner();
            const { programAddress, eventAuthority } = getProgramConfig();
            const distribution = input.distribution.trim();
            const mint = input.mint.trim();
            const recipient = input.recipient.trim();
            const originalPayer = input.originalPayer.trim();
            const tokenProgram = normalizeTokenProgram(input.tokenProgram);
            const [recipientAccount] = deriveDirectRecipientPda(distribution, recipient, programAddress);
            const distributionVault = deriveAta(distribution, mint, tokenProgram);
            const recipientTokenAccount = deriveAta(recipient, mint, tokenProgram);
            const authorityTokenAccount = deriveAta(txSigner.address, mint, tokenProgram);

            const instruction = getRevokeDirectRecipientInstruction(
                {
                    authority: txSigner,
                    authorityTokenAccount,
                    distribution: asAddress(distribution),
                    distributionVault,
                    eventAuthority,
                    mint: asAddress(mint),
                    originalPayer: asAddress(originalPayer),
                    recipient: asAddress(recipient),
                    recipientAccount: asAddress(recipientAccount),
                    recipientTokenAccount,
                    revokeMode: input.revokeMode,
                    tokenProgram,
                },
                { programAddress },
            );

            const signature = await sendRewardTransaction([instruction], txSigner, 'Revoke Direct Recipient', {
                distribution,
                mint,
                recipient,
            });
            return { signature };
        },
        onError,
        onSuccess,
    });

    const revokeMerkleClaim = useMutation({
        mutationFn: async (input: RevokeMerkleClaimInput): Promise<RewardsMutationResult> => {
            const txSigner = requireSigner();
            const { programAddress, eventAuthority } = getProgramConfig();
            const distribution = input.distribution.trim();
            const mint = input.mint.trim();
            const claimant = input.claimant.trim();
            const tokenProgram = normalizeTokenProgram(input.tokenProgram);
            const [claimAccount] = deriveMerkleClaimPda(distribution, claimant, programAddress);
            const [revocationMarker] = deriveRevocationPda(distribution, claimant, programAddress);
            const distributionVault = deriveAta(distribution, mint, tokenProgram);
            const claimantTokenAccount = deriveAta(claimant, mint, tokenProgram);
            const authorityTokenAccount = deriveAta(txSigner.address, mint, tokenProgram);

            const instruction = getRevokeMerkleClaimInstruction(
                {
                    authority: txSigner,
                    authorityTokenAccount,
                    claimAccount: asAddress(claimAccount),
                    claimant: asAddress(claimant),
                    claimantTokenAccount,
                    distribution: asAddress(distribution),
                    distributionVault,
                    eventAuthority,
                    mint: asAddress(mint),
                    payer: txSigner,
                    proof: input.proof.map(node => [...node]),
                    revocationMarker: asAddress(revocationMarker),
                    revokeMode: input.revokeMode,
                    schedule: input.schedule,
                    tokenProgram,
                    totalAmount: input.totalAmount,
                },
                { programAddress },
            );

            const signature = await sendRewardTransaction([instruction], txSigner, 'Revoke Merkle Claim', {
                claimant,
                distribution,
                mint,
                totalAmount: String(input.totalAmount),
            });
            return { signature };
        },
        onError,
        onSuccess,
    });

    return {
        addDirectRecipient,
        claimDirect,
        claimMerkle,
        closeDirectDistribution,
        closeDirectRecipient,
        closeMerkleClaim,
        closeMerkleDistribution,
        createDirectDistribution,
        createMerkleDistribution,
        revokeDirectRecipient,
        revokeMerkleClaim,
    };
}
