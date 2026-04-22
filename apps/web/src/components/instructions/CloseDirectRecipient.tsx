'use client';

import { useState } from 'react';
import { Badge } from '@solana/design-system/badge';
import { getCloseDirectRecipientInstruction } from '@solana/rewards-client';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useWallet } from '@/contexts/WalletContext';
import { useSendTx } from '@/hooks/useSendTx';
import { deriveDirectRecipientPda } from '@/lib/pdas';
import { firstValidationError, validateAddress } from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, SendButton } from './shared';
import { asAddress, getProgramConfig } from './common';

export function CloseDirectRecipient() {
    const { account, createSigner } = useWallet();
    const { send, sending, signature, error, reset } = useSendTx();
    const { defaultDistribution } = useSavedValues();
    const [distribution, setDistribution] = useState('');
    const [originalPayer, setOriginalPayer] = useState('');
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        reset();
        setFormError(null);

        const signer = createSigner();
        if (!signer) return;

        const validationError = firstValidationError(
            validateAddress(distribution, 'Distribution address'),
            validateAddress(originalPayer, 'Original payer'),
        );
        if (validationError) {
            setFormError(validationError);
            return;
        }

        const { programAddress, eventAuthority } = getProgramConfig();
        const [recipientAccount] = deriveDirectRecipientPda(distribution, signer.address, programAddress);

        const ix = getCloseDirectRecipientInstruction(
            {
                recipient: signer.address,
                originalPayer: asAddress(originalPayer),
                distribution: asAddress(distribution),
                recipientAccount: asAddress(recipientAccount),
                eventAuthority,
            },
            { programAddress },
        );

        await send([ix], {
            action: 'Close Direct Recipient',
            values: { distribution },
        });
    };

    return (
        <form onSubmit={e => void handleSubmit(e)} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <div>
                <Badge variant="info">
                    Recipient signer is your connected wallet: {account?.address ?? 'Not connected'}
                </Badge>
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
                label="Original Payer"
                value={originalPayer}
                onChange={setOriginalPayer}
                placeholder="Address receiving recipient-account rent refund"
                required
            />
            <SendButton sending={sending} />
            <TxResult signature={signature} error={formError ?? error} />
        </form>
    );
}
