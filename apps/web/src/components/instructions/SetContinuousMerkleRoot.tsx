'use client';

import { useState } from 'react';
import { getSetContinuousMerkleRootInstruction } from '@solana/rewards-client';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useWallet } from '@/contexts/WalletContext';
import { useSendTx } from '@/hooks/useSendTx';
import {
    firstValidationError,
    parseBigIntValue,
    parseByteArray32,
    validateAddress,
    validatePositiveInteger,
} from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, SendButton } from './shared';
import { asAddress, getProgramConfig } from './common';

export function SetContinuousMerkleRoot() {
    const { createSigner } = useWallet();
    const { send, sending, signature, error, reset } = useSendTx();
    const { defaultRewardPool, rememberRewardPool } = useSavedValues();
    const [rewardPool, setRewardPool] = useState('');
    const [merkleRoot, setMerkleRoot] = useState('');
    const [rootVersion, setRootVersion] = useState('1');
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        reset();
        setFormError(null);

        const signer = createSigner();
        if (!signer) return;

        const rootResult = parseByteArray32(merkleRoot, 'Merkle root');
        if (!rootResult.ok) {
            setFormError(rootResult.error);
            return;
        }

        const validationError = firstValidationError(
            validateAddress(rewardPool, 'Reward pool'),
            validatePositiveInteger(rootVersion, 'Root version'),
        );

        if (validationError) {
            setFormError(validationError);
            return;
        }

        const { programAddress, eventAuthority } = getProgramConfig();

        const ix = getSetContinuousMerkleRootInstruction(
            {
                authority: signer,
                rewardPool: asAddress(rewardPool),
                eventAuthority,
                merkleRoot: rootResult.value,
                rootVersion: parseBigIntValue(rootVersion),
            },
            { programAddress },
        );

        const txSignature = await send([ix], {
            action: 'Set Continuous Merkle Root',
            values: { rewardPool, rootVersion },
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
            <FormField
                label="Merkle Root"
                value={merkleRoot}
                onChange={setMerkleRoot}
                placeholder="32-byte hex (with or without 0x)"
                hint="Also accepts JSON number array with 32 bytes"
                required
            />
            <FormField
                label="Root Version"
                value={rootVersion}
                onChange={setRootVersion}
                type="number"
                placeholder="Must be strictly increasing"
                required
            />
            <SendButton sending={sending} />
            <TxResult signature={signature} error={formError ?? error} />
        </form>
    );
}
