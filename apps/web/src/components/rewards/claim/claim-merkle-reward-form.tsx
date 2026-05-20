import { useState } from 'react';
import { useWallet } from '@solana/connector/react';
import { Badge } from '@solana/design-system/badge';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useRewardsMutations } from '@/hooks/use-rewards-mutations';
import { useTokenFormDefaults } from '@/hooks/use-token-form-defaults';
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
import { INITIAL_VESTING_SCHEDULE, type VestingScheduleState } from '@/lib/vesting-schedule';
import {
    buildVestingSchedule,
    FormField,
    SendButton,
    TextAreaField,
    VestingScheduleField,
} from '../shared/reward-form-fields';

interface ClaimMerkleRewardFormProps {
    hideKnownFields?: boolean;
    initialDistribution?: string;
    initialMint?: string;
    initialProof?: string;
    initialSchedule?: VestingScheduleState;
    initialTotalAmount?: string;
    onSuccess?: () => void;
    submitLabel?: string;
}

export function ClaimMerkleRewardForm({
    hideKnownFields = false,
    initialDistribution = '',
    initialMint = '',
    initialProof = '',
    initialSchedule = INITIAL_VESTING_SCHEDULE,
    initialTotalAmount = '',
    onSuccess,
    submitLabel,
}: ClaimMerkleRewardFormProps) {
    const { account } = useWallet();
    const { claimMerkle } = useRewardsMutations();
    const { defaultDistribution, defaultMint, rememberDistribution, rememberMint } = useSavedValues();
    const [distribution, setDistribution] = useState(initialDistribution);
    const { clusterMint, mint, setMint, setTokenProgram, tokenProgram } = useTokenFormDefaults(initialMint);
    const [totalAmount, setTotalAmount] = useState(initialTotalAmount);
    const [amount, setAmount] = useState('0');
    const [proof, setProof] = useState(initialProof);
    const [schedule, setSchedule] = useState<VestingScheduleState>(initialSchedule);
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        claimMerkle.reset();
        setFormError(null);

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
            validateAddress(distribution, 'Drop address'),
            validateAddress(mint, 'Mint address'),
            validatePositiveInteger(totalAmount, 'Total amount'),
            validateNonNegativeInteger(amount, 'Amount'),
            validateOptionalAddress(tokenProgram, 'Token program'),
        );
        if (validationError) {
            setFormError(validationError);
            return;
        }

        const result = await claimMerkle
            .mutateAsync({
                amount: parseBigIntValue(amount),
                distribution,
                mint,
                proof: proofResult.value,
                schedule: scheduleResult.value,
                tokenProgram,
                totalAmount: parseBigIntValue(totalAmount),
            })
            .catch(() => null);
        if (!result) return;

        rememberDistribution(distribution);
        rememberMint(mint);
        onSuccess?.();
    };

    return (
        <form onSubmit={e => void handleSubmit(e)} className="flex flex-col gap-4">
            <div>
                <Badge variant="info">Claiming wallet: {account ?? 'Not connected'}</Badge>
            </div>
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
                        autoFillValue={defaultMint || clusterMint}
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
                </>
            )}
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
                placeholder="Token program address"
            />
            {!hideKnownFields && (
                <>
                    <VestingScheduleField value={schedule} onChange={setSchedule} />
                    <TextAreaField
                        label="Proof"
                        value={proof}
                        onChange={setProof}
                        placeholder="JSON arrays or one 32-byte hex node per line"
                    />
                </>
            )}
            <SendButton sending={claimMerkle.isPending} label={submitLabel} />
            <TxResult signature={claimMerkle.data?.signature} error={formError ?? claimMerkle.error} />
        </form>
    );
}
