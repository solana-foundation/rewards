'use client';

import { useState } from 'react';
import { getRevokeMerkleClaimInstruction, RevokeMode } from '@solana/rewards';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useWallet } from '@/contexts/WalletContext';
import { useSendTx } from '@/hooks/useSendTx';
import { deriveAta, deriveMerkleClaimPda, deriveRevocationPda } from '@/lib/pdas';
import {
    firstValidationError,
    parseBigIntValue,
    parseMerkleProof,
    validateAddress,
    validateOptionalAddress,
    validatePositiveInteger,
} from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import {
    buildVestingSchedule,
    FormField,
    INITIAL_VESTING_SCHEDULE,
    RevokeModeField,
    SendButton,
    TextAreaField,
    VestingScheduleField,
    type VestingScheduleState,
} from './shared';
import { asAddress, getProgramConfig, normalizeTokenProgramInput } from './common';

export function RevokeMerkleClaim() {
    const { createSigner } = useWallet();
    const { send, sending, signature, error, reset } = useSendTx();
    const { defaultDistribution, defaultMint, rememberDistribution, rememberMint } = useSavedValues();
    const [distribution, setDistribution] = useState('');
    const [mint, setMint] = useState('');
    const [claimant, setClaimant] = useState('');
    const [totalAmount, setTotalAmount] = useState('');
    const [proof, setProof] = useState('');
    const [tokenProgram, setTokenProgram] = useState('');
    const [revokeMode, setRevokeMode] = useState<RevokeMode>(RevokeMode.NonVested);
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
            validateAddress(claimant, 'Claimant address'),
            validatePositiveInteger(totalAmount, 'Total amount'),
            validateOptionalAddress(tokenProgram, 'Token program'),
        );

        if (validationError) {
            setFormError(validationError);
            return;
        }

        const { programAddress, eventAuthority } = getProgramConfig();
        const tokenProgramAddress = normalizeTokenProgramInput(tokenProgram);

        const [claimAccount] = deriveMerkleClaimPda(distribution, claimant, programAddress);
        const [revocationMarker] = deriveRevocationPda(distribution, claimant, programAddress);
        const distributionVault = deriveAta(distribution, mint, tokenProgramAddress);
        const claimantTokenAccount = deriveAta(claimant, mint, tokenProgramAddress);
        const authorityTokenAccount = deriveAta(signer.address, mint, tokenProgramAddress);

        const ix = getRevokeMerkleClaimInstruction(
            {
                authority: signer,
                payer: signer,
                distribution: asAddress(distribution),
                claimAccount: asAddress(claimAccount),
                revocationMarker: asAddress(revocationMarker),
                claimant: asAddress(claimant),
                mint: asAddress(mint),
                distributionVault,
                claimantTokenAccount,
                authorityTokenAccount,
                tokenProgram: tokenProgramAddress,
                eventAuthority,
                revokeMode,
                totalAmount: parseBigIntValue(totalAmount),
                schedule: scheduleResult.value,
                proof: proofResult.value,
            },
            { programAddress },
        );

        const txSignature = await send([ix], {
            action: 'Revoke Merkle Claim',
            values: { distribution, mint, claimant, totalAmount },
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
                label="Claimant Address"
                value={claimant}
                onChange={setClaimant}
                placeholder="Wallet to revoke"
                required
            />
            <FormField
                label="Total Allocation Amount"
                value={totalAmount}
                onChange={setTotalAmount}
                type="number"
                required
            />
            <FormField
                label="Token Program"
                value={tokenProgram}
                onChange={setTokenProgram}
                placeholder="Defaults to Tokenkeg..."
            />
            <RevokeModeField value={revokeMode} onChange={setRevokeMode} />
            <VestingScheduleField value={schedule} onChange={setSchedule} />
            <TextAreaField
                label="Merkle Proof"
                value={proof}
                onChange={setProof}
                placeholder="JSON arrays or one 32-byte hex node per line"
                hint="Proof for the claimant leaf containing claimant + total amount + schedule"
            />
            <SendButton sending={sending} />
            <TxResult signature={signature} error={formError ?? error} />
        </form>
    );
}
