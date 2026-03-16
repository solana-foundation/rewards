'use client';

import { useState } from 'react';
import { Badge } from '@solana/design-system/badge';
import { getClaimContinuousMerkleInstruction } from '@solana/rewards-client';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useWallet } from '@/contexts/WalletContext';
import { useSendTx } from '@/hooks/useSendTx';
import { deriveAta, deriveMerkleClaimPda, deriveRevocationPda, normalizeTokenProgram } from '@/lib/pdas';
import {
    firstValidationError,
    parseBigIntValue,
    parseMerkleProof,
    validateAddress,
    validateNonNegativeInteger,
    validateOptionalAddress,
    validatePositiveInteger,
} from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, SendButton, TextAreaField } from './shared';
import { asAddress, getProgramConfig } from './common';

export function ClaimContinuousMerkle() {
    const { account, createSigner } = useWallet();
    const { send, sending, signature, error, reset } = useSendTx();
    const { defaultRewardPool, defaultRewardMint, rememberRewardPool, rememberRewardMint } = useSavedValues();
    const [rewardPool, setRewardPool] = useState('');
    const [rewardMint, setRewardMint] = useState('');
    const [rootVersion, setRootVersion] = useState('');
    const [cumulativeAmount, setCumulativeAmount] = useState('');
    const [amount, setAmount] = useState('0');
    const [proof, setProof] = useState('');
    const [rewardTokenProgram, setRewardTokenProgram] = useState('');
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        reset();
        setFormError(null);

        const signer = createSigner();
        if (!signer) return;

        const proofResult = parseMerkleProof(proof);
        if (!proofResult.ok) {
            setFormError(proofResult.error);
            return;
        }

        const validationError = firstValidationError(
            validateAddress(rewardPool, 'Reward pool'),
            validateAddress(rewardMint, 'Reward mint'),
            validatePositiveInteger(rootVersion, 'Root version'),
            validatePositiveInteger(cumulativeAmount, 'Cumulative amount'),
            validateNonNegativeInteger(amount, 'Amount'),
            validateOptionalAddress(rewardTokenProgram, 'Reward token program'),
        );

        if (validationError) {
            setFormError(validationError);
            return;
        }

        const { programAddress, eventAuthority } = getProgramConfig();
        const rewardTokenProgramAddress = normalizeTokenProgram(rewardTokenProgram);

        const [claimAccount, claimBump] = deriveMerkleClaimPda(rewardPool, signer.address, programAddress);
        const [revocationMarker] = deriveRevocationPda(rewardPool, signer.address, programAddress);
        const rewardVault = deriveAta(rewardPool, rewardMint, rewardTokenProgramAddress);
        const userRewardTokenAccount = deriveAta(signer.address, rewardMint, rewardTokenProgramAddress);

        const ix = getClaimContinuousMerkleInstruction(
            {
                payer: signer,
                user: signer,
                rewardPool: asAddress(rewardPool),
                claimAccount: asAddress(claimAccount),
                revocationMarker: asAddress(revocationMarker),
                rewardMint: asAddress(rewardMint),
                rewardVault,
                userRewardTokenAccount,
                rewardTokenProgram: rewardTokenProgramAddress,
                eventAuthority,
                claimBump,
                rootVersion: parseBigIntValue(rootVersion),
                cumulativeAmount: parseBigIntValue(cumulativeAmount),
                amount: parseBigIntValue(amount),
                proof: proofResult.value,
            },
            { programAddress },
        );

        const txSignature = await send([ix], {
            action: 'Claim Continuous Merkle',
            values: { rewardPool, rewardMint, rootVersion, amount },
        });

        if (txSignature) {
            rememberRewardPool(rewardPool);
            rememberRewardMint(rewardMint);
        }
    };

    return (
        <form onSubmit={e => void handleSubmit(e)} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <div>
                <Badge variant="info">User signer is your connected wallet: {account?.address ?? 'Not connected'}</Badge>
            </div>
            <FormField
                label="Reward Pool"
                value={rewardPool}
                onChange={setRewardPool}
                autoFillValue={defaultRewardPool}
                onAutoFill={setRewardPool}
                placeholder="Reward pool PDA"
                required
            />
            <FormField
                label="Reward Mint"
                value={rewardMint}
                onChange={setRewardMint}
                autoFillValue={defaultRewardMint}
                onAutoFill={setRewardMint}
                placeholder="Reward mint"
                required
            />
            <FormField label="Root Version" value={rootVersion} onChange={setRootVersion} type="number" required />
            <FormField
                label="Cumulative Amount"
                value={cumulativeAmount}
                onChange={setCumulativeAmount}
                type="number"
                required
            />
            <FormField
                label="Amount (0 for full claimable delta)"
                value={amount}
                onChange={setAmount}
                type="number"
                required
            />
            <FormField
                label="Reward Token Program"
                value={rewardTokenProgram}
                onChange={setRewardTokenProgram}
                placeholder="Defaults to Tokenkeg..."
            />
            <TextAreaField
                label="Merkle Proof"
                value={proof}
                onChange={setProof}
                placeholder='JSON arrays or one 32-byte hex node per line'
                hint='Proof nodes for (reward_pool, user, root_version, cumulative_amount) leaf'
            />
            <SendButton sending={sending} />
            <TxResult signature={signature} error={formError ?? error} />
        </form>
    );
}
