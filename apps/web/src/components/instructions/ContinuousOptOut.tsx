'use client';

import { useState } from 'react';
import { Badge } from '@solana/design-system/badge';
import { getContinuousOptOutInstruction } from '@solana/rewards-client';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useWallet } from '@/contexts/WalletContext';
import { useSendTx } from '@/hooks/useSendTx';
import { deriveAta, deriveUserRewardPda, normalizeTokenProgram } from '@/lib/pdas';
import { firstValidationError, validateAddress, validateOptionalAddress } from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, SendButton } from './shared';
import { asAddress, getProgramConfig } from './common';

export function ContinuousOptOut() {
    const { account, createSigner } = useWallet();
    const { send, sending, signature, error, reset } = useSendTx();
    const { defaultRewardPool, defaultTrackedMint, defaultRewardMint, rememberRewardPool, rememberTrackedMint, rememberRewardMint } =
        useSavedValues();
    const [rewardPool, setRewardPool] = useState('');
    const [trackedMint, setTrackedMint] = useState('');
    const [rewardMint, setRewardMint] = useState('');
    const [trackedTokenProgram, setTrackedTokenProgram] = useState('');
    const [rewardTokenProgram, setRewardTokenProgram] = useState('');
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
            validateAddress(rewardMint, 'Reward mint'),
            validateOptionalAddress(trackedTokenProgram, 'Tracked token program'),
            validateOptionalAddress(rewardTokenProgram, 'Reward token program'),
        );

        if (validationError) {
            setFormError(validationError);
            return;
        }

        const { programAddress, eventAuthority } = getProgramConfig();
        const trackedTokenProgramAddress = normalizeTokenProgram(trackedTokenProgram);
        const rewardTokenProgramAddress = normalizeTokenProgram(rewardTokenProgram);

        const [userRewardAccount] = deriveUserRewardPda(rewardPool, signer.address, programAddress);
        const userTrackedTokenAccount = deriveAta(signer.address, trackedMint, trackedTokenProgramAddress);
        const rewardVault = deriveAta(rewardPool, rewardMint, rewardTokenProgramAddress);
        const userRewardTokenAccount = deriveAta(signer.address, rewardMint, rewardTokenProgramAddress);

        const ix = getContinuousOptOutInstruction(
            {
                user: signer,
                rewardPool: asAddress(rewardPool),
                userRewardAccount: asAddress(userRewardAccount),
                userTrackedTokenAccount,
                rewardVault,
                userRewardTokenAccount,
                trackedMint: asAddress(trackedMint),
                rewardMint: asAddress(rewardMint),
                trackedTokenProgram: trackedTokenProgramAddress,
                rewardTokenProgram: rewardTokenProgramAddress,
                eventAuthority,
            },
            { programAddress },
        );

        const txSignature = await send([ix], {
            action: 'Continuous Opt Out',
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
            <div>
                <Badge variant="info">User signer is your connected wallet: {account?.address ?? 'Not connected'}</Badge>
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
                label="Reward Mint"
                value={rewardMint}
                onChange={setRewardMint}
                autoFillValue={defaultRewardMint}
                onAutoFill={setRewardMint}
                placeholder="Reward mint"
                required
            />
            <FormField
                label="Tracked Token Program"
                value={trackedTokenProgram}
                onChange={setTrackedTokenProgram}
                placeholder="Defaults to Tokenkeg..."
            />
            <FormField
                label="Reward Token Program"
                value={rewardTokenProgram}
                onChange={setRewardTokenProgram}
                placeholder="Defaults to Tokenkeg..."
            />
            <SendButton sending={sending} />
            <TxResult signature={signature} error={formError ?? error} />
        </form>
    );
}
