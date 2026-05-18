import { useState } from 'react';
import { RevokeMode } from '@solana/rewards';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useRewardsMutations } from '@/hooks/use-rewards-mutations';
import { parseProofDropClaimBundle } from '@/lib/proof-drop-bundle';
import { firstValidationError, parseBigIntValue, validateAddress, validateOptionalAddress } from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import {
    buildVestingSchedule,
    FormField,
    RevokeModeField,
    SendButton,
    TextAreaField,
} from '../shared/reward-form-fields';

interface RevokeMerkleClaimFormProps {
    hideKnownFields?: boolean;
    initialDistribution?: string;
    initialMint?: string;
    onSuccess?: () => void;
    submitLabel?: string;
}

export function RevokeMerkleClaimForm({
    hideKnownFields = false,
    initialDistribution = '',
    initialMint = '',
    onSuccess,
    submitLabel,
}: RevokeMerkleClaimFormProps) {
    const { revokeMerkleClaim } = useRewardsMutations();
    const { defaultDistribution, defaultMint, rememberDistribution, rememberMint } = useSavedValues();
    const [distribution, setDistribution] = useState(initialDistribution);
    const [mint, setMint] = useState(initialMint);
    const [claimant, setClaimant] = useState('');
    const [bundle, setBundle] = useState('');
    const [tokenProgram, setTokenProgram] = useState('');
    const [revokeMode, setRevokeMode] = useState<RevokeMode>(RevokeMode.NonVested);
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        revokeMerkleClaim.reset();
        setFormError(null);

        const validationError = firstValidationError(
            validateAddress(distribution, 'Drop address'),
            validateAddress(mint, 'Mint address'),
            claimant.trim() ? validateAddress(claimant, 'Recipient address') : null,
            validateOptionalAddress(tokenProgram, 'Token program'),
        );

        if (validationError) {
            setFormError(validationError);
            return;
        }

        const claimResult = parseProofDropClaimBundle(bundle, {
            claimant,
            distribution,
            requireDistribution: true,
        });
        if (!claimResult.ok) {
            setFormError(claimResult.error);
            return;
        }

        const scheduleResult = buildVestingSchedule(claimResult.value.schedule);
        if (!scheduleResult.ok) {
            setFormError(scheduleResult.error);
            return;
        }

        const result = await revokeMerkleClaim
            .mutateAsync({
                claimant: claimResult.value.recipient,
                distribution,
                mint,
                proof: claimResult.value.proof,
                revokeMode,
                schedule: scheduleResult.value,
                tokenProgram,
                totalAmount: parseBigIntValue(claimResult.value.totalAmount),
            })
            .catch(() => null);
        if (!result) return;

        rememberDistribution(distribution);
        rememberMint(mint);
        onSuccess?.();
    };

    return (
        <form onSubmit={e => void handleSubmit(e)} className="flex flex-col gap-4">
            {!hideKnownFields && (
                <>
                    <FormField
                        label="Drop Address"
                        value={distribution}
                        onChange={setDistribution}
                        autoFillValue={defaultDistribution}
                        onAutoFill={setDistribution}
                        placeholder="Proof-based drop address"
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
                </>
            )}
            <FormField
                label="Recipient Address"
                value={claimant}
                onChange={setClaimant}
                placeholder="Recipient wallet to revoke"
            />
            <TextAreaField
                label="Recipient Proof Bundle"
                value={bundle}
                onChange={setBundle}
                placeholder='{"kind":"proof-drop","distribution":"...","recipients":[...]}'
                rows={8}
                required
            />
            <FormField
                label="Token Program"
                value={tokenProgram}
                onChange={setTokenProgram}
                placeholder="Defaults to Tokenkeg..."
            />
            <RevokeModeField value={revokeMode} onChange={setRevokeMode} />
            <SendButton sending={revokeMerkleClaim.isPending} label={submitLabel} />
            <TxResult signature={revokeMerkleClaim.data?.signature} error={formError ?? revokeMerkleClaim.error} />
        </form>
    );
}
