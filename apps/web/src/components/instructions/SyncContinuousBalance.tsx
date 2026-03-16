'use client';

import { useState } from 'react';
import { getSyncContinuousBalanceInstruction } from '@solana/rewards-client';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useSendTx } from '@/hooks/useSendTx';
import { deriveAta, deriveUserRewardPda, normalizeTokenProgram } from '@/lib/pdas';
import { firstValidationError, validateAddress, validateOptionalAddress } from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, SendButton } from './shared';
import { asAddress, getProgramConfig } from './common';

export function SyncContinuousBalance() {
    const { send, sending, signature, error, reset } = useSendTx();
    const { defaultRewardPool, defaultTrackedMint, rememberRewardPool, rememberTrackedMint } = useSavedValues();
    const [rewardPool, setRewardPool] = useState('');
    const [user, setUser] = useState('');
    const [trackedMint, setTrackedMint] = useState('');
    const [trackedTokenProgram, setTrackedTokenProgram] = useState('');
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        reset();
        setFormError(null);

        const validationError = firstValidationError(
            validateAddress(rewardPool, 'Reward pool'),
            validateAddress(user, 'User address'),
            validateAddress(trackedMint, 'Tracked mint'),
            validateOptionalAddress(trackedTokenProgram, 'Tracked token program'),
        );

        if (validationError) {
            setFormError(validationError);
            return;
        }

        const { programAddress, eventAuthority } = getProgramConfig();
        const trackedTokenProgramAddress = normalizeTokenProgram(trackedTokenProgram);

        const [userRewardAccount] = deriveUserRewardPda(rewardPool, user, programAddress);
        const userTrackedTokenAccount = deriveAta(user, trackedMint, trackedTokenProgramAddress);

        const ix = getSyncContinuousBalanceInstruction(
            {
                rewardPool: asAddress(rewardPool),
                userRewardAccount: asAddress(userRewardAccount),
                user: asAddress(user),
                userTrackedTokenAccount,
                trackedMint: asAddress(trackedMint),
                trackedTokenProgram: trackedTokenProgramAddress,
                eventAuthority,
            },
            { programAddress },
        );

        const txSignature = await send([ix], {
            action: 'Sync Continuous Balance',
            values: { rewardPool, trackedMint, user },
        });

        if (txSignature) {
            rememberRewardPool(rewardPool);
            rememberTrackedMint(trackedMint);
        }
    };

    return (
        <form onSubmit={e => void handleSubmit(e)} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <FormField
                label="Reward Pool"
                value={rewardPool}
                onChange={setRewardPool}
                autoFillValue={defaultRewardPool}
                onAutoFill={setRewardPool}
                placeholder="Reward pool PDA"
                required
            />
            <FormField label="User Address" value={user} onChange={setUser} placeholder="User wallet" required />
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
