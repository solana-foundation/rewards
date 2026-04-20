'use client';

import { useState } from 'react';
import { generateKeyPairSigner } from '@solana/kit';
import { getCreateDirectDistributionInstruction } from '@solana/rewards-client';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useWallet } from '@/contexts/WalletContext';
import { useSendTx } from '@/hooks/useSendTx';
import { deriveAta, deriveDirectDistributionPda } from '@/lib/pdas';
import {
    parseBigIntValue,
    validateAddress,
    validateInteger,
    validateOptionalAddress,
    firstValidationError,
} from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, SelectField, SendButton } from './shared';
import { asAddress, getProgramConfig, normalizeTokenProgramInput } from './common';

export function CreateDirectDistribution() {
    const { createSigner } = useWallet();
    const { send, sending, signature, error, reset } = useSendTx();
    const { defaultMint, rememberDistribution, rememberMint } = useSavedValues();
    const [mint, setMint] = useState('');
    const [revocable, setRevocable] = useState('0');
    const [clawbackTs, setClawbackTs] = useState('0');
    const [tokenProgram, setTokenProgram] = useState('');
    const [generatedSeed, setGeneratedSeed] = useState('');
    const [generatedDistribution, setGeneratedDistribution] = useState('');
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        reset();
        setFormError(null);

        const signer = createSigner();
        if (!signer) return;

        const validationError = firstValidationError(
            validateAddress(mint, 'Mint address'),
            validateInteger(clawbackTs, 'Clawback timestamp'),
            validateOptionalAddress(tokenProgram, 'Token program'),
        );
        if (validationError) {
            setFormError(validationError);
            return;
        }

        const { programAddress, eventAuthority } = getProgramConfig();
        const tokenProgramAddress = normalizeTokenProgramInput(tokenProgram);
        const seedSigner = await generateKeyPairSigner();
        setGeneratedSeed(seedSigner.address);

        const [distribution, bump] = deriveDirectDistributionPda(
            mint,
            signer.address,
            seedSigner.address,
            programAddress,
        );
        setGeneratedDistribution(distribution);

        const distributionVault = deriveAta(distribution, mint, tokenProgramAddress);

        const ix = getCreateDirectDistributionInstruction(
            {
                payer: signer,
                authority: signer,
                seeds: seedSigner,
                distribution: asAddress(distribution),
                mint: asAddress(mint),
                distributionVault,
                tokenProgram: tokenProgramAddress,
                eventAuthority,
                bump,
                revocable: Number(revocable),
                clawbackTs: parseBigIntValue(clawbackTs),
            },
            { programAddress },
        );

        const txSignature = await send([ix], {
            action: 'Create Direct Distribution',
            values: { distribution, mint },
        });

        if (txSignature) {
            rememberDistribution(distribution);
            rememberMint(mint);
        }
    };

    return (
        <form onSubmit={e => void handleSubmit(e)} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <FormField
                label="Mint Address"
                value={mint}
                onChange={setMint}
                autoFillValue={defaultMint}
                onAutoFill={setMint}
                placeholder="SPL token mint"
                required
            />
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
                hint="Authority can close distribution after this timestamp"
                required
            />
            <FormField
                label="Token Program"
                value={tokenProgram}
                onChange={setTokenProgram}
                placeholder="Defaults to Tokenkeg..."
                hint="Use Token-2022 program address for Token-2022 mints"
            />
            {generatedSeed && (
                <FormField
                    label="Generated Seed"
                    value={generatedSeed}
                    onChange={() => {}}
                    readOnly
                    hint="Random signer used to derive distribution PDA"
                />
            )}
            {generatedDistribution && (
                <FormField
                    label="Generated Distribution PDA"
                    value={generatedDistribution}
                    onChange={() => {}}
                    readOnly
                    hint="Saved as default distribution after success"
                />
            )}
            <SendButton sending={sending} />
            <TxResult signature={signature} error={formError ?? error} />
        </form>
    );
}
