'use client';

import { useState } from 'react';
import { getRevokeContinuousUserInstruction, RevokeMode } from '@solana/rewards-client';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useWallet } from '@/contexts/WalletContext';
import { useSendTx } from '@/hooks/useSendTx';
import { deriveAta, deriveRevocationPda, deriveUserRewardPda, normalizeTokenProgram } from '@/lib/pdas';
import { firstValidationError, validateAddress, validateOptionalAddress } from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, RevokeModeField, SendButton } from './shared';
import { asAddress, getProgramConfig } from './common';

export function RevokeContinuousUser() {
    const { account, createSigner } = useWallet();
    const { send, sending, signature, error, reset } = useSendTx();
    const {
        defaultRewardPool,
        defaultTrackedMint,
        defaultRewardMint,
        rememberRewardPool,
        rememberTrackedMint,
        rememberRewardMint,
    } = useSavedValues();
    const [rewardPool, setRewardPool] = useState('');
    const [user, setUser] = useState('');
    const [rentDestination, setRentDestination] = useState('');
    const [trackedMint, setTrackedMint] = useState('');
    const [rewardMint, setRewardMint] = useState('');
    const [trackedTokenProgram, setTrackedTokenProgram] = useState('');
    const [rewardTokenProgram, setRewardTokenProgram] = useState('');
    const [revokeMode, setRevokeMode] = useState<RevokeMode>(RevokeMode.NonVested);
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        reset();
        setFormError(null);

        const signer = createSigner();
        if (!signer) return;

        const validationError = firstValidationError(
            validateAddress(rewardPool, 'Reward pool'),
            validateAddress(user, 'User address'),
            validateAddress(trackedMint, 'Tracked mint'),
            validateAddress(rewardMint, 'Reward mint'),
            validateOptionalAddress(rentDestination, 'Rent destination'),
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

        const [userRewardAccount] = deriveUserRewardPda(rewardPool, user, programAddress);
        const [revocationMarker] = deriveRevocationPda(rewardPool, user, programAddress);
        const userTrackedTokenAccount = deriveAta(user, trackedMint, trackedTokenProgramAddress);
        const rewardVault = deriveAta(rewardPool, rewardMint, rewardTokenProgramAddress);
        const userRewardTokenAccount = deriveAta(user, rewardMint, rewardTokenProgramAddress);
        const authorityRewardTokenAccount = deriveAta(signer.address, rewardMint, rewardTokenProgramAddress);

        const ix = getRevokeContinuousUserInstruction(
            {
                authority: signer,
                payer: signer,
                rewardPool: asAddress(rewardPool),
                userRewardAccount: asAddress(userRewardAccount),
                revocationMarker: asAddress(revocationMarker),
                user: asAddress(user),
                rentDestination: asAddress((rentDestination || signer.address).trim()),
                userTrackedTokenAccount,
                rewardVault,
                userRewardTokenAccount,
                authorityRewardTokenAccount,
                trackedMint: asAddress(trackedMint),
                rewardMint: asAddress(rewardMint),
                trackedTokenProgram: trackedTokenProgramAddress,
                rewardTokenProgram: rewardTokenProgramAddress,
                eventAuthority,
                revokeMode,
            },
            { programAddress },
        );

        const txSignature = await send([ix], {
            action: 'Revoke Continuous User',
            values: { rewardPool, trackedMint, rewardMint, user },
        });

        if (txSignature) {
            rememberRewardPool(rewardPool);
            rememberTrackedMint(trackedMint);
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
            <FormField label="User Address" value={user} onChange={setUser} placeholder="User to revoke" required />
            <FormField
                label="Rent Destination"
                value={rentDestination}
                onChange={setRentDestination}
                placeholder={account?.address ?? 'Defaults to connected wallet'}
                hint="Recipient for closed user reward account rent"
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
            <RevokeModeField value={revokeMode} onChange={setRevokeMode} />
            <SendButton sending={sending} />
            <TxResult signature={signature} error={formError ?? error} />
        </form>
    );
}
