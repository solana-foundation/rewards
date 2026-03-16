'use client';

import { useState } from 'react';
import { getSetContinuousBalanceInstruction } from '@solana/rewards-client';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useWallet } from '@/contexts/WalletContext';
import { useSendTx } from '@/hooks/useSendTx';
import { deriveUserRewardPda } from '@/lib/pdas';
import { firstValidationError, parseBigIntValue, validateAddress, validateNonNegativeInteger } from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, SendButton } from './shared';
import { asAddress, getProgramConfig } from './common';

export function SetContinuousBalance() {
    const { createSigner } = useWallet();
    const { send, sending, signature, error, reset } = useSendTx();
    const { defaultRewardPool, rememberRewardPool } = useSavedValues();
    const [rewardPool, setRewardPool] = useState('');
    const [user, setUser] = useState('');
    const [balance, setBalance] = useState('0');
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
            validateNonNegativeInteger(balance, 'Balance'),
        );

        if (validationError) {
            setFormError(validationError);
            return;
        }

        const { programAddress, eventAuthority } = getProgramConfig();
        const [userRewardAccount] = deriveUserRewardPda(rewardPool, user, programAddress);

        const ix = getSetContinuousBalanceInstruction(
            {
                authority: signer,
                rewardPool: asAddress(rewardPool),
                userRewardAccount: asAddress(userRewardAccount),
                user: asAddress(user),
                eventAuthority,
                balance: parseBigIntValue(balance),
            },
            { programAddress },
        );

        const txSignature = await send([ix], {
            action: 'Set Continuous Balance',
            values: { rewardPool, user },
        });

        if (txSignature) {
            rememberRewardPool(rewardPool);
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
            <FormField label="Balance (base units)" value={balance} onChange={setBalance} type="number" required />
            <SendButton sending={sending} />
            <TxResult signature={signature} error={formError ?? error} />
        </form>
    );
}
