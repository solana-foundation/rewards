'use client';

import { useState } from 'react';
import { getCloseDirectDistributionInstruction } from '@solana/rewards';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useWallet } from '@/contexts/WalletContext';
import { useSendTx } from '@/hooks/useSendTx';
import { deriveAta, normalizeTokenProgram } from '@/lib/pdas';
import { firstValidationError, validateAddress, validateOptionalAddress } from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, SendButton } from './shared';
import { asAddress, getProgramConfig } from './common';

export function CloseDirectDistribution() {
    const { createSigner } = useWallet();
    const { send, sending, signature, error, reset } = useSendTx();
    const { defaultDistribution, defaultMint, rememberDistribution, rememberMint } = useSavedValues();
    const [distribution, setDistribution] = useState('');
    const [mint, setMint] = useState('');
    const [tokenProgram, setTokenProgram] = useState('');
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        reset();
        setFormError(null);

        const signer = createSigner();
        if (!signer) return;

        const validationError = firstValidationError(
            validateAddress(distribution, 'Distribution address'),
            validateAddress(mint, 'Mint address'),
            validateOptionalAddress(tokenProgram, 'Token program'),
        );
        if (validationError) {
            setFormError(validationError);
            return;
        }

        const { programAddress, eventAuthority } = getProgramConfig();
        const tokenProgramAddress = normalizeTokenProgram(tokenProgram);
        const distributionVault = deriveAta(distribution, mint, tokenProgramAddress);
        const authorityTokenAccount = deriveAta(signer.address, mint, tokenProgramAddress);

        const ix = getCloseDirectDistributionInstruction(
            {
                authority: signer,
                distribution: asAddress(distribution),
                mint: asAddress(mint),
                distributionVault,
                authorityTokenAccount,
                tokenProgram: tokenProgramAddress,
                eventAuthority,
            },
            { programAddress },
        );

        const txSignature = await send([ix], {
            action: 'Close Direct Distribution',
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
                label="Distribution Address"
                value={distribution}
                onChange={setDistribution}
                autoFillValue={defaultDistribution}
                onAutoFill={setDistribution}
                placeholder="Direct distribution PDA"
                required
            />
            <FormField
                label="Mint Address"
                value={mint}
                onChange={setMint}
                autoFillValue={defaultMint}
                onAutoFill={setMint}
                placeholder="SPL token mint"
                required
            />
            <FormField
                label="Token Program"
                value={tokenProgram}
                onChange={setTokenProgram}
                placeholder="Defaults to Tokenkeg..."
            />
            <SendButton sending={sending} />
            <TxResult signature={signature} error={formError ?? error} />
        </form>
    );
}
