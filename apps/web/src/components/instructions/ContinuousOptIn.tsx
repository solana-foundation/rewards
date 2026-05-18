'use client';

import { useState } from 'react';
import { Badge } from '@solana/design-system/badge';
import { getContinuousOptInInstruction } from '@solana/rewards';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useWallet } from '@/contexts/WalletContext';
import { useSendTx } from '@/hooks/useSendTx';
import { deriveAta, deriveRevocationPda, deriveUserRewardPda, normalizeTokenProgram } from '@/lib/pdas';
import { firstValidationError, validateAddress, validateOptionalAddress } from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, SendButton } from './shared';
import { asAddress, getProgramConfig } from './common';

export function ContinuousOptIn() {
    const { account, createSigner } = useWallet();
    const { send, sending, signature, error, reset } = useSendTx();
    const { defaultRewardPool, defaultTrackedMint, rememberRewardPool, rememberTrackedMint } = useSavedValues();
    const [rewardPool, setRewardPool] = useState('');
    const [trackedMint, setTrackedMint] = useState('');
    const [trackedTokenProgram, setTrackedTokenProgram] = useState('');
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        reset();
        setFormError(null);

        const signer = createSigner();
        if (!signer) return;

        const validationError = firstValidationError(
            validateAddress(rewardPool, 'Reward pool'),
            validateAddress(trackedMint, 'Tracked mint'),
            validateOptionalAddress(trackedTokenProgram, 'Tracked token program'),
        );
        if (validationError) {
            setFormError(validationError);
            return;
        }

        const { programAddress, eventAuthority } = getProgramConfig();
        const trackedTokenProgramAddress = normalizeTokenProgram(trackedTokenProgram);

        const [userRewardAccount, bump] = deriveUserRewardPda(rewardPool, signer.address, programAddress);
        const [revocationMarker] = deriveRevocationPda(rewardPool, signer.address, programAddress);
        const userTrackedTokenAccount = deriveAta(signer.address, trackedMint, trackedTokenProgramAddress);

        const ix = getContinuousOptInInstruction(
            {
                payer: signer,
                user: signer,
                rewardPool: asAddress(rewardPool),
                userRewardAccount: asAddress(userRewardAccount),
                revocationMarker: asAddress(revocationMarker),
                userTrackedTokenAccount,
                trackedMint: asAddress(trackedMint),
                trackedTokenProgram: trackedTokenProgramAddress,
                eventAuthority,
                bump,
            },
            { programAddress },
        );

        const txSignature = await send([ix], {
            action: 'Continuous Opt In',
            values: { rewardPool, trackedMint, user: signer.address },
        });

        if (txSignature) {
            rememberRewardPool(rewardPool);
            rememberTrackedMint(trackedMint);
        }
    };

    return (
        <form onSubmit={e => void handleSubmit(e)} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <div>
                <Badge variant="info">
                    User signer is your connected wallet: {account?.address ?? 'Not connected'}
                </Badge>
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
                label="Tracked Mint"
                value={trackedMint}
                onChange={setTrackedMint}
                autoFillValue={defaultTrackedMint}
                onAutoFill={setTrackedMint}
                placeholder="Tracked mint"
                required
            />
            <FormField
                label="Tracked Token Program"
                value={trackedTokenProgram}
                onChange={setTrackedTokenProgram}
                placeholder="Defaults to Tokenkeg..."
            />
            <SendButton sending={sending} />
            <TxResult signature={signature} error={formError ?? error} />
        </form>
    );
}
