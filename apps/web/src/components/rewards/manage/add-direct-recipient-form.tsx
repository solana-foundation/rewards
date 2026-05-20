import { useState } from 'react';
import { Button } from '@solana/design-system';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useRewardsMutations } from '@/hooks/use-rewards-mutations';
import { useTokenFormDefaults } from '@/hooks/use-token-form-defaults';
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
} from '../shared/reward-form-fields';

interface AddDirectRecipientFormProps {
    initialDistribution?: string;
    initialMint?: string;
    onSuccess?: () => void;
    submitLabel?: string;
}

const DAY_SECONDS = 86_400;

type RecipientTemplate = {
    amount: string;
    label: string;
    schedule: () => VestingScheduleState;
};

function timestampDaysFromNow(days: number) {
    return String(Math.floor(Date.now() / 1000) + days * DAY_SECONDS);
}

const RECIPIENT_TEMPLATES: readonly RecipientTemplate[] = [
    {
        amount: '1000',
        label: 'Immediate',
        schedule: () => INITIAL_VESTING_SCHEDULE,
    },
    {
        amount: '1000',
        label: '30d Linear',
        schedule: () => ({
            cliffTs: '',
            endTs: timestampDaysFromNow(30),
            kind: 'Linear',
            startTs: timestampDaysFromNow(0),
        }),
    },
    {
        amount: '1000',
        label: 'Cliff 90d',
        schedule: () => ({
            cliffTs: timestampDaysFromNow(7),
            endTs: timestampDaysFromNow(90),
            kind: 'CliffLinear',
            startTs: timestampDaysFromNow(0),
        }),
    },
] as const;

export function AddDirectRecipientForm({
    initialDistribution = '',
    initialMint = '',
    onSuccess,
    submitLabel,
}: AddDirectRecipientFormProps) {
    const { addDirectRecipient } = useRewardsMutations();
    const { defaultDistribution, defaultMint, rememberDistribution, rememberMint } = useSavedValues();
    const [distribution, setDistribution] = useState(initialDistribution);
    const { clusterMint, mint, setMint, setTokenProgram, tokenProgram } = useTokenFormDefaults(initialMint);
    const [recipient, setRecipient] = useState('');
    const [amount, setAmount] = useState('');
    const [schedule, setSchedule] = useState<VestingScheduleState>(INITIAL_VESTING_SCHEDULE);
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        addDirectRecipient.reset();
        setFormError(null);

        const scheduleResult = buildVestingSchedule(schedule);
        if (!scheduleResult.ok) {
            setFormError(scheduleResult.error);
            return;
        }

        const validationError = firstValidationError(
            validateAddress(distribution, 'Reward address'),
            validateAddress(mint, 'Mint address'),
            validateAddress(recipient, 'Recipient address'),
            validatePositiveInteger(amount, 'Amount'),
            validateOptionalAddress(tokenProgram, 'Token program'),
        );
        if (validationError) {
            setFormError(validationError);
            return;
        }

        const result = await addDirectRecipient
            .mutateAsync({
                amount: parseBigIntValue(amount),
                distribution,
                mint,
                recipient,
                schedule: scheduleResult.value,
                tokenProgram,
            })
            .catch(() => null);
        if (!result) return;

        rememberDistribution(distribution);
        rememberMint(mint);
        onSuccess?.();
    };

    const applyTemplate = (template: RecipientTemplate) => {
        setAmount(template.amount);
        setSchedule(template.schedule());
    };

    return (
        <form onSubmit={e => void handleSubmit(e)} className="flex flex-col gap-4">
            <div className="flex flex-wrap gap-2">
                {RECIPIENT_TEMPLATES.map(template => (
                    <Button
                        key={template.label}
                        type="button"
                        variant="secondary"
                        size="sm"
                        onClick={() => applyTemplate(template)}
                    >
                        {template.label}
                    </Button>
                ))}
            </div>
            <FormField
                label="Reward Address"
                value={distribution}
                onChange={setDistribution}
                autoFillValue={defaultDistribution}
                onAutoFill={setDistribution}
                placeholder="Reward campaign address"
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
                placeholder="Token program address"
            />
            <VestingScheduleField value={schedule} onChange={setSchedule} />
            <SendButton sending={addDirectRecipient.isPending} label={submitLabel} />
            <TxResult signature={addDirectRecipient.data?.signature} error={formError ?? addDirectRecipient.error} />
        </form>
    );
}
