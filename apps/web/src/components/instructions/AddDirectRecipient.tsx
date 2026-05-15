'use client';

import { useState } from 'react';
import { getAddDirectRecipientInstruction } from '@solana/rewards';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useWallet } from '@/contexts/WalletContext';
import { useSendTx } from '@/hooks/useSendTx';
import { deriveAta, deriveDirectRecipientPda } from '@/lib/pdas';
import {
    firstValidationError,
    parseBigIntValue,
    validateAddress,
    validateOptionalAddress,
    validatePositiveInteger,
} from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import {
    buildVestingSchedule,
    FormField,
    INITIAL_VESTING_SCHEDULE,
    SendButton,
    VestingScheduleField,
    type VestingScheduleState,
} from './shared';
import { asAddress, getProgramConfig, normalizeTokenProgramInput } from './common';

export function AddDirectRecipient() {
    const { createSigner } = useWallet();
    const { send, sending, signature, error, reset } = useSendTx();
    const { defaultDistribution, defaultMint, rememberDistribution, rememberMint } = useSavedValues();
    const [distribution, setDistribution] = useState('');
    const [mint, setMint] = useState('');
    const [recipient, setRecipient] = useState('');
    const [amount, setAmount] = useState('');
    const [tokenProgram, setTokenProgram] = useState('');
    const [schedule, setSchedule] = useState<VestingScheduleState>(INITIAL_VESTING_SCHEDULE);
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        reset();
        setFormError(null);

        const signer = createSigner();
        if (!signer) return;

        const scheduleResult = buildVestingSchedule(schedule);
        if (!scheduleResult.ok) {
            setFormError(scheduleResult.error);
            return;
        }

        const validationError = firstValidationError(
            validateAddress(distribution, 'Distribution address'),
            validateAddress(mint, 'Mint address'),
            validateAddress(recipient, 'Recipient address'),
            validatePositiveInteger(amount, 'Amount'),
            validateOptionalAddress(tokenProgram, 'Token program'),
        );
        if (validationError) {
            setFormError(validationError);
            return;
        }

        const { programAddress, eventAuthority } = getProgramConfig();
        const tokenProgramAddress = normalizeTokenProgramInput(tokenProgram);

        const [recipientAccount, bump] = deriveDirectRecipientPda(distribution, recipient, programAddress);
        const distributionVault = deriveAta(distribution, mint, tokenProgramAddress);
        const authorityTokenAccount = deriveAta(signer.address, mint, tokenProgramAddress);

        const ix = getAddDirectRecipientInstruction(
            {
                payer: signer,
                authority: signer,
                distribution: asAddress(distribution),
                recipientAccount: asAddress(recipientAccount),
                recipient: asAddress(recipient),
                mint: asAddress(mint),
                distributionVault,
                authorityTokenAccount,
                tokenProgram: tokenProgramAddress,
                eventAuthority,
                bump,
                amount: parseBigIntValue(amount),
                schedule: scheduleResult.value,
            },
            { programAddress },
        );

        const txSignature = await send([ix], {
            action: 'Add Direct Recipient',
            values: { distribution, mint, recipient, amount },
        });

        if (txSignature) {
            rememberDistribution(distribution);
            rememberMint(mint);
        }
    };

    return (
        <form onSubmit={e => void handleSubmit(e)} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
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
                label="Recipient Address"
                value={recipient}
                onChange={setRecipient}
                placeholder="Wallet receiving allocation"
                required
            />
            <FormField
                label="Amount (base units)"
                value={amount}
                onChange={setAmount}
                type="number"
                placeholder="e.g. 1000000"
                required
            />
            <FormField
                label="Token Program"
                value={tokenProgram}
                onChange={setTokenProgram}
                placeholder="Defaults to Tokenkeg..."
            />
            <VestingScheduleField value={schedule} onChange={setSchedule} />
            <SendButton sending={sending} />
            <TxResult signature={signature} error={formError ?? error} />
        </form>
    );
}
