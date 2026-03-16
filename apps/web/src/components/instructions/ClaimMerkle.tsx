'use client';

import { useState } from 'react';
import { Badge } from '@solana/design-system/badge';
import { getClaimMerkleInstruction } from '@solana/rewards-client';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useWallet } from '@/contexts/WalletContext';
import { useSendTx } from '@/hooks/useSendTx';
import { deriveAta, deriveMerkleClaimPda, deriveRevocationPda } from '@/lib/pdas';
import {
    firstValidationError,
    parseBigIntValue,
    parseMerkleProof,
    validateAddress,
    validateNonNegativeInteger,
    validateOptionalAddress,
    validatePositiveInteger,
} from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import {
    buildVestingSchedule,
    FormField,
    INITIAL_VESTING_SCHEDULE,
    SendButton,
    TextAreaField,
    VestingScheduleField,
    type VestingScheduleState,
} from './shared';
import { asAddress, getProgramConfig, normalizeTokenProgramInput } from './common';

export function ClaimMerkle() {
    const { account, createSigner } = useWallet();
    const { send, sending, signature, error, reset } = useSendTx();
    const { defaultDistribution, defaultMint, rememberDistribution, rememberMint } = useSavedValues();
    const [distribution, setDistribution] = useState('');
    const [mint, setMint] = useState('');
    const [totalAmount, setTotalAmount] = useState('');
    const [amount, setAmount] = useState('0');
    const [proof, setProof] = useState('');
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

        const proofResult = parseMerkleProof(proof);
        if (!proofResult.ok) {
            setFormError(proofResult.error);
            return;
        }

        const validationError = firstValidationError(
            validateAddress(distribution, 'Distribution address'),
            validateAddress(mint, 'Mint address'),
            validatePositiveInteger(totalAmount, 'Total amount'),
            validateNonNegativeInteger(amount, 'Amount'),
            validateOptionalAddress(tokenProgram, 'Token program'),
        );
        if (validationError) {
            setFormError(validationError);
            return;
        }

        const { programAddress, eventAuthority } = getProgramConfig();
        const tokenProgramAddress = normalizeTokenProgramInput(tokenProgram);

        const [claimAccount, claimBump] = deriveMerkleClaimPda(distribution, signer.address, programAddress);
        const [revocationMarker] = deriveRevocationPda(distribution, signer.address, programAddress);
        const distributionVault = deriveAta(distribution, mint, tokenProgramAddress);
        const claimantTokenAccount = deriveAta(signer.address, mint, tokenProgramAddress);

        const ix = getClaimMerkleInstruction(
            {
                payer: signer,
                claimant: signer,
                distribution: asAddress(distribution),
                claimAccount: asAddress(claimAccount),
                revocationMarker: asAddress(revocationMarker),
                mint: asAddress(mint),
                distributionVault,
                claimantTokenAccount,
                tokenProgram: tokenProgramAddress,
                eventAuthority,
                claimBump,
                totalAmount: parseBigIntValue(totalAmount),
                amount: parseBigIntValue(amount),
                schedule: scheduleResult.value,
                proof: proofResult.value,
            },
            { programAddress },
        );

        const txSignature = await send([ix], {
            action: 'Claim Merkle',
            values: { distribution, mint, totalAmount, amount },
        });

        if (txSignature) {
            rememberDistribution(distribution);
            rememberMint(mint);
        }
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
                label="Total Allocation Amount"
                value={totalAmount}
                onChange={setTotalAmount}
                type="number"
                placeholder="Leaf total allocation"
                required
            />
            <FormField
                label="Claim Amount (0 for max claimable delta)"
                value={amount}
                onChange={setAmount}
                type="number"
                required
            />
            <FormField
                label="Token Program"
                value={tokenProgram}
                onChange={setTokenProgram}
                placeholder="Defaults to Tokenkeg..."
            />
            <VestingScheduleField value={schedule} onChange={setSchedule} />
            <TextAreaField
                label="Merkle Proof"
                value={proof}
                onChange={setProof}
                placeholder='JSON arrays or one 32-byte hex node per line'
                hint='Example JSON: [[1,2,...32],[...]]'
            />
            <SendButton sending={sending} />
            <TxResult signature={signature} error={formError ?? error} />
        </form>
    );
}
