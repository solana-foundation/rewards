'use client';

import { useState } from 'react';
import { getRevokeDirectRecipientInstruction, RevokeMode } from '@solana/rewards';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useWallet } from '@/contexts/WalletContext';
import { useSendTx } from '@/hooks/useSendTx';
import { deriveAta, deriveDirectRecipientPda } from '@/lib/pdas';
import { firstValidationError, validateAddress, validateOptionalAddress } from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, RevokeModeField, SendButton } from './shared';
import { asAddress, getProgramConfig, normalizeTokenProgramInput } from './common';

export function RevokeDirectRecipient() {
    const { createSigner } = useWallet();
    const { send, sending, signature, error, reset } = useSendTx();
    const { defaultDistribution, defaultMint, rememberDistribution, rememberMint } = useSavedValues();
    const [distribution, setDistribution] = useState('');
    const [mint, setMint] = useState('');
    const [recipient, setRecipient] = useState('');
    const [originalPayer, setOriginalPayer] = useState('');
    const [tokenProgram, setTokenProgram] = useState('');
    const [revokeMode, setRevokeMode] = useState<RevokeMode>(RevokeMode.NonVested);
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
            validateAddress(recipient, 'Recipient address'),
            validateAddress(originalPayer, 'Original payer'),
            validateOptionalAddress(tokenProgram, 'Token program'),
        );

        if (validationError) {
            setFormError(validationError);
            return;
        }

        const { programAddress, eventAuthority } = getProgramConfig();
        const tokenProgramAddress = normalizeTokenProgramInput(tokenProgram);

        const [recipientAccount] = deriveDirectRecipientPda(distribution, recipient, programAddress);
        const distributionVault = deriveAta(distribution, mint, tokenProgramAddress);
        const recipientTokenAccount = deriveAta(recipient, mint, tokenProgramAddress);
        const authorityTokenAccount = deriveAta(signer.address, mint, tokenProgramAddress);

        const ix = getRevokeDirectRecipientInstruction(
            {
                authority: signer,
                distribution: asAddress(distribution),
                recipientAccount: asAddress(recipientAccount),
                recipient: asAddress(recipient),
                originalPayer: asAddress(originalPayer),
                mint: asAddress(mint),
                distributionVault,
                recipientTokenAccount,
                authorityTokenAccount,
                tokenProgram: tokenProgramAddress,
                eventAuthority,
                revokeMode,
            },
            { programAddress },
        );

        const txSignature = await send([ix], {
            action: 'Revoke Direct Recipient',
            values: { distribution, mint, recipient },
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
                label="Recipient Address"
                value={recipient}
                onChange={setRecipient}
                placeholder="Recipient wallet to revoke"
                required
            />
            <FormField
                label="Original Payer"
                value={originalPayer}
                onChange={setOriginalPayer}
                placeholder="Original payer receiving rent refund"
                required
            />
            <FormField
                label="Token Program"
                value={tokenProgram}
                onChange={setTokenProgram}
                placeholder="Defaults to Tokenkeg..."
            />
            <RevokeModeField value={revokeMode} onChange={setRevokeMode} />
            <SendButton sending={sending} />
            <TxResult signature={signature} error={formError ?? error} />
        </form>
    );
}
