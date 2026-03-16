'use client';

import { useState } from 'react';
import { generateKeyPairSigner } from '@solana/kit';
import { getCreateMerkleDistributionInstruction } from '@solana/rewards-client';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useWallet } from '@/contexts/WalletContext';
import { useSendTx } from '@/hooks/useSendTx';
import { deriveAta, deriveMerkleDistributionPda } from '@/lib/pdas';
import {
    firstValidationError,
    parseBigIntValue,
    parseByteArray32,
    validateAddress,
    validateInteger,
    validateOptionalAddress,
    validatePositiveInteger,
} from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, SelectField, SendButton } from './shared';
import { asAddress, getProgramConfig, normalizeTokenProgramInput } from './common';

export function CreateMerkleDistribution() {
    const { createSigner } = useWallet();
    const { send, sending, signature, error, reset } = useSendTx();
    const { defaultMint, rememberDistribution, rememberMint } = useSavedValues();
    const [mint, setMint] = useState('');
    const [revocable, setRevocable] = useState('0');
    const [amount, setAmount] = useState('');
    const [totalAmount, setTotalAmount] = useState('');
    const [clawbackTs, setClawbackTs] = useState('0');
    const [merkleRoot, setMerkleRoot] = useState('');
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

        const rootResult = parseByteArray32(merkleRoot, 'Merkle root');
        if (!rootResult.ok) {
            setFormError(rootResult.error);
            return;
        }

        const validationError = firstValidationError(
            validateAddress(mint, 'Mint address'),
            validatePositiveInteger(amount, 'Initial funded amount'),
            validatePositiveInteger(totalAmount, 'Total merkle amount'),
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

        const [distribution, bump] = deriveMerkleDistributionPda(mint, signer.address, seedSigner.address, programAddress);
        setGeneratedDistribution(distribution);

        const distributionVault = deriveAta(distribution, mint, tokenProgramAddress);
        const authorityTokenAccount = deriveAta(signer.address, mint, tokenProgramAddress);

        const ix = getCreateMerkleDistributionInstruction(
            {
                payer: signer,
                authority: signer,
                seeds: seedSigner,
                distribution: asAddress(distribution),
                mint: asAddress(mint),
                distributionVault,
                authorityTokenAccount,
                tokenProgram: tokenProgramAddress,
                eventAuthority,
                bump,
                revocable: Number(revocable),
                amount: parseBigIntValue(amount),
                merkleRoot: rootResult.value,
                totalAmount: parseBigIntValue(totalAmount),
                clawbackTs: parseBigIntValue(clawbackTs),
            },
            { programAddress },
        );

        const txSignature = await send([ix], {
            action: 'Create Merkle Distribution',
            values: { distribution, mint, amount, totalAmount },
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
                label="Initial Funded Amount"
                value={amount}
                onChange={setAmount}
                type="number"
                placeholder="Amount transferred into vault on creation"
                required
            />
            <FormField
                label="Total Merkle Amount"
                value={totalAmount}
                onChange={setTotalAmount}
                type="number"
                placeholder="Sum of all leaf allocations"
                required
            />
            <FormField
                label="Clawback Timestamp (i64)"
                value={clawbackTs}
                onChange={setClawbackTs}
                type="number"
                placeholder="Unix timestamp"
                required
            />
            <FormField
                label="Merkle Root"
                value={merkleRoot}
                onChange={setMerkleRoot}
                placeholder="32-byte hex (with or without 0x)"
                hint="Also accepts JSON number array with 32 bytes"
                required
            />
            <FormField
                label="Token Program"
                value={tokenProgram}
                onChange={setTokenProgram}
                placeholder="Defaults to Tokenkeg..."
            />
            {generatedSeed && (
                <FormField
                    label="Generated Seed"
                    value={generatedSeed}
                    onChange={() => {}}
                    readOnly
                    hint="Random signer used to derive merkle distribution PDA"
                />
            )}
            {generatedDistribution && (
                <FormField
                    label="Generated Distribution PDA"
                    value={generatedDistribution}
                    onChange={() => {}}
                    readOnly
                />
            )}
            <SendButton sending={sending} />
            <TxResult signature={signature} error={formError ?? error} />
        </form>
    );
}
