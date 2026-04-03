'use client';

import { useState } from 'react';
import { getCloseContinuousPoolInstruction } from '@solana/rewards-client';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useWallet } from '@/contexts/WalletContext';
import { useSendTx } from '@/hooks/useSendTx';
import { deriveAta, normalizeTokenProgram } from '@/lib/pdas';
import { firstValidationError, validateAddress, validateOptionalAddress } from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, SendButton } from './shared';
import { asAddress, getProgramConfig } from './common';

export function CloseContinuousPool() {
    const { createSigner } = useWallet();
    const { send, sending, signature, error, reset } = useSendTx();
    const { defaultRewardPool, defaultRewardMint, rememberRewardPool, rememberRewardMint } = useSavedValues();
    const [rewardPool, setRewardPool] = useState('');
    const [rewardMint, setRewardMint] = useState('');
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
            validateAddress(rewardMint, 'Reward mint'),
            validateOptionalAddress(rewardTokenProgram, 'Reward token program'),
        );

        if (validationError) {
            setFormError(validationError);
            return;
        }

        const { programAddress, eventAuthority } = getProgramConfig();
        const rewardTokenProgramAddress = normalizeTokenProgram(rewardTokenProgram);
        const rewardVault = deriveAta(rewardPool, rewardMint, rewardTokenProgramAddress);
        const authorityTokenAccount = deriveAta(signer.address, rewardMint, rewardTokenProgramAddress);

        const ix = getCloseContinuousPoolInstruction(
            {
                authority: signer,
                rewardPool: asAddress(rewardPool),
                rewardMint: asAddress(rewardMint),
                rewardVault,
                authorityTokenAccount,
                rewardTokenProgram: rewardTokenProgramAddress,
                eventAuthority,
            },
            { programAddress },
        );

        const txSignature = await send([ix], {
            action: 'Close Continuous Pool',
            values: { rewardPool, rewardMint },
        });

        if (txSignature) {
            rememberRewardPool(rewardPool);
            rememberRewardMint(rewardMint);
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
