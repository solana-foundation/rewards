'use client';

import { useState } from 'react';
import { BalanceSource, getCreateContinuousPoolInstruction } from '@solana/rewards-client';
import { generateKeyPairSigner } from '@solana/kit';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useWallet } from '@/contexts/WalletContext';
import { useSendTx } from '@/hooks/useSendTx';
import { deriveAta, deriveRewardPoolPda, normalizeTokenProgram } from '@/lib/pdas';
import {
    firstValidationError,
    parseBigIntValue,
    validateAddress,
    validateInteger,
    validateOptionalAddress,
} from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { BalanceSourceField, FormField, SelectField, SendButton } from './shared';
import { asAddress, getProgramConfig } from './common';

export function CreateContinuousPool() {
    const { createSigner } = useWallet();
    const { send, sending, signature, error, reset } = useSendTx();
    const { defaultTrackedMint, defaultRewardMint, rememberRewardPool, rememberTrackedMint, rememberRewardMint } =
        useSavedValues();
    const [trackedMint, setTrackedMint] = useState('');
    const [rewardMint, setRewardMint] = useState('');
    const [rewardTokenProgram, setRewardTokenProgram] = useState('');
    const [balanceSource, setBalanceSource] = useState<BalanceSource>(BalanceSource.OnChain);
    const [revocable, setRevocable] = useState('0');
    const [clawbackTs, setClawbackTs] = useState('0');
    const [generatedSeed, setGeneratedSeed] = useState('');
    const [generatedPool, setGeneratedPool] = useState('');
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        reset();
        setFormError(null);

        const signer = createSigner();
        if (!signer) return;

        const validationError = firstValidationError(
            validateAddress(trackedMint, 'Tracked mint'),
            validateAddress(rewardMint, 'Reward mint'),
            validateInteger(clawbackTs, 'Clawback timestamp'),
            validateOptionalAddress(rewardTokenProgram, 'Reward token program'),
        );

        if (validationError) {
            setFormError(validationError);
            return;
        }

        const { programAddress, eventAuthority } = getProgramConfig();
        const rewardTokenProgramAddress = normalizeTokenProgram(rewardTokenProgram);

        const seedSigner = await generateKeyPairSigner();
        setGeneratedSeed(seedSigner.address);

        const [rewardPool, bump] = deriveRewardPoolPda(
            rewardMint,
            trackedMint,
            signer.address,
            seedSigner.address,
            programAddress,
        );
        setGeneratedPool(rewardPool);

        const rewardVault = deriveAta(rewardPool, rewardMint, rewardTokenProgramAddress);

        const ix = getCreateContinuousPoolInstruction(
            {
                payer: signer,
                authority: signer,
                seed: seedSigner,
                rewardPool: asAddress(rewardPool),
                trackedMint: asAddress(trackedMint),
                rewardMint: asAddress(rewardMint),
                rewardVault,
                rewardTokenProgram: rewardTokenProgramAddress,
                eventAuthority,
                bump,
                balanceSource,
                revocable: Number(revocable),
                clawbackTs: parseBigIntValue(clawbackTs),
            },
            { programAddress },
        );

        const txSignature = await send([ix], {
            action: 'Create Continuous Pool',
            values: { rewardPool, trackedMint, rewardMint },
        });

        if (txSignature) {
            rememberRewardPool(rewardPool);
            rememberTrackedMint(trackedMint);
            rememberRewardMint(rewardMint);
        }
    };

    return (
        <form onSubmit={e => void handleSubmit(e)} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <FormField
                label="Tracked Mint"
                value={trackedMint}
                onChange={setTrackedMint}
                autoFillValue={defaultTrackedMint}
                onAutoFill={setTrackedMint}
                placeholder="Mint used for balance tracking"
                required
            />
            <FormField
                label="Reward Mint"
                value={rewardMint}
                onChange={setRewardMint}
                autoFillValue={defaultRewardMint}
                onAutoFill={setRewardMint}
                placeholder="Mint distributed as reward"
                required
            />
            <FormField
                label="Reward Token Program"
                value={rewardTokenProgram}
                onChange={setRewardTokenProgram}
                placeholder="Defaults to Tokenkeg..."
            />
            <BalanceSourceField value={balanceSource} onChange={setBalanceSource} />
            <SelectField
                label="Revocable"
                value={revocable}
                onChange={setRevocable}
                options={[
                    { label: 'No (0)', value: '0' },
                    { label: 'Yes (1)', value: '1' },
                ]}
            />
            <FormField
                label="Clawback Timestamp (i64)"
                value={clawbackTs}
                onChange={setClawbackTs}
                type="number"
                placeholder="Unix timestamp"
                required
            />
            {generatedSeed && (
                <FormField
                    label="Generated Seed"
                    value={generatedSeed}
                    onChange={() => {}}
                    readOnly
                    hint="Random signer used to derive pool PDA"
                />
            )}
            {generatedPool && (
                <FormField
                    label="Generated Reward Pool PDA"
                    value={generatedPool}
                    onChange={() => {}}
                    readOnly
                />
            )}
            <SendButton sending={sending} />
            <TxResult signature={signature} error={formError ?? error} />
        </form>
    );
}
