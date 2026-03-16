'use client';

import { useState } from 'react';
import { Badge } from '@solana/design-system/badge';
import { getClaimDirectInstruction } from '@solana/rewards-client';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useWallet } from '@/contexts/WalletContext';
import { useSendTx } from '@/hooks/useSendTx';
import { deriveAta, deriveDirectRecipientPda } from '@/lib/pdas';
import {
    firstValidationError,
    parseBigIntValue,
    validateAddress,
    validateNonNegativeInteger,
    validateOptionalAddress,
} from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, SendButton } from './shared';
import { asAddress, getProgramConfig, normalizeTokenProgramInput } from './common';

export function ClaimDirect() {
    const { account, createSigner } = useWallet();
    const { send, sending, signature, error, reset } = useSendTx();
    const { defaultDistribution, defaultMint, rememberDistribution, rememberMint } = useSavedValues();
    const [distribution, setDistribution] = useState('');
    const [mint, setMint] = useState('');
    const [amount, setAmount] = useState('0');
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
            validateNonNegativeInteger(amount, 'Amount'),
            validateOptionalAddress(tokenProgram, 'Token program'),
        );
        if (validationError) {
            setFormError(validationError);
            return;
        }

        const { programAddress, eventAuthority } = getProgramConfig();
        const tokenProgramAddress = normalizeTokenProgramInput(tokenProgram);

        const [recipientAccount] = deriveDirectRecipientPda(distribution, signer.address, programAddress);
        const distributionVault = deriveAta(distribution, mint, tokenProgramAddress);
        const recipientTokenAccount = deriveAta(signer.address, mint, tokenProgramAddress);

        const ix = getClaimDirectInstruction(
            {
                recipient: signer,
                distribution: asAddress(distribution),
                recipientAccount: asAddress(recipientAccount),
                mint: asAddress(mint),
                distributionVault,
                recipientTokenAccount,
                tokenProgram: tokenProgramAddress,
                eventAuthority,
                amount: parseBigIntValue(amount),
            },
            { programAddress },
        );

        const txSignature = await send([ix], {
            action: 'Claim Direct',
            values: { distribution, mint, amount },
        });

        if (txSignature) {
            rememberDistribution(distribution);
            rememberMint(mint);
        }
    };

    return (
        <form onSubmit={e => void handleSubmit(e)} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <div>
                <Badge variant="info">Recipient signer is your connected wallet: {account?.address ?? 'Not connected'}</Badge>
            </div>
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
                label="Amount (0 for max claimable)"
                value={amount}
                onChange={setAmount}
                type="number"
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
