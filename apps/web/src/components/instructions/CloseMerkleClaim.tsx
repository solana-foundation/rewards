'use client';

import { useState } from 'react';
import { Badge } from '@solana/design-system/badge';
import { getCloseMerkleClaimInstruction } from '@solana/rewards-client';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useWallet } from '@/contexts/WalletContext';
import { useSendTx } from '@/hooks/useSendTx';
import { deriveMerkleClaimPda } from '@/lib/pdas';
import { validateAddress } from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, SendButton } from './shared';
import { asAddress, getProgramConfig } from './common';

export function CloseMerkleClaim() {
    const { account, createSigner } = useWallet();
    const { send, sending, signature, error, reset } = useSendTx();
    const { defaultDistribution } = useSavedValues();
    const [distribution, setDistribution] = useState('');
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        reset();
        setFormError(null);

        const signer = createSigner();
        if (!signer) return;

        const validationError = validateAddress(distribution, 'Distribution address');
        if (validationError) {
            setFormError(validationError);
            return;
        }

        const { programAddress, eventAuthority } = getProgramConfig();
        const [claimAccount] = deriveMerkleClaimPda(distribution, signer.address, programAddress);

        const ix = getCloseMerkleClaimInstruction(
            {
                claimant: signer,
                distribution: asAddress(distribution),
                claimAccount: asAddress(claimAccount),
                eventAuthority,
            },
            { programAddress },
        );

        await send([ix], {
            action: 'Close Merkle Claim',
            values: { distribution },
        });
    };

    return (
        <form onSubmit={e => void handleSubmit(e)} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <div>
                <Badge variant="info">Claimant signer is your connected wallet: {account?.address ?? 'Not connected'}</Badge>
            </div>
            <FormField
                label="Distribution Address"
                value={distribution}
                onChange={setDistribution}
                autoFillValue={defaultDistribution}
                onAutoFill={setDistribution}
                placeholder="Merkle distribution PDA"
                required
            />
            <SendButton sending={sending} />
            <TxResult signature={signature} error={formError ?? error} />
        </form>
    );
}
