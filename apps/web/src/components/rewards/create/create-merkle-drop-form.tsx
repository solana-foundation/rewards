import { useState } from 'react';
import { Button } from '@solana/design-system';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useRewardsMutations } from '@/hooks/use-rewards-mutations';
import { useTokenFormDefaults } from '@/hooks/use-token-form-defaults';
import {
    buildProofDropBundle,
    parseProofDropRecipients,
    proofDropBundleText,
    type BuiltProofDropBundle,
} from '@/lib/proof-drop-bundle';
import {
    firstValidationError,
    parseBigIntValue,
    validateAddress,
    validateInteger,
    validateOptionalAddress,
    validatePositiveInteger,
} from '@/lib/validation';
import { INITIAL_VESTING_SCHEDULE, type VestingScheduleState } from '@/lib/vesting-schedule';
import { TxResult } from '@/components/TxResult';
import { FormField, SelectField, SendButton, TextAreaField, VestingScheduleField } from '../shared/reward-form-fields';

const DAY_SECONDS = 86_400;
const SAMPLE_RECIPIENTS = [
    '7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgS5U',
    'SysvarRent111111111111111111111111111111111',
    'SysvarC1ock11111111111111111111111111111111',
] as const;

type ProofDropTemplate = {
    amount: string;
    clawbackDays: number;
    label: string;
    recipients: string;
    revocable: string;
    schedule: () => VestingScheduleState;
};

function timestampDaysFromNow(days: number) {
    return String(Math.floor(Date.now() / 1000) + days * DAY_SECONDS);
}

const PROOF_DROP_TEMPLATES: readonly ProofDropTemplate[] = [
    {
        amount: '3500',
        clawbackDays: 0,
        label: 'Immediate Drop',
        recipients: `${SAMPLE_RECIPIENTS[0]},1000\n${SAMPLE_RECIPIENTS[1]},1500\n${SAMPLE_RECIPIENTS[2]},1000`,
        revocable: '0',
        schedule: () => INITIAL_VESTING_SCHEDULE,
    },
    {
        amount: '3000',
        clawbackDays: 90,
        label: '30d Vest',
        recipients: `${SAMPLE_RECIPIENTS[0]},1000\n${SAMPLE_RECIPIENTS[1]},1000\n${SAMPLE_RECIPIENTS[2]},1000`,
        revocable: '1',
        schedule: () => ({
            cliffTs: '',
            endTs: timestampDaysFromNow(30),
            kind: 'Linear',
            startTs: timestampDaysFromNow(0),
        }),
    },
    {
        amount: '3000',
        clawbackDays: 120,
        label: 'Cliff Drop',
        recipients: `${SAMPLE_RECIPIENTS[0]},1000\n${SAMPLE_RECIPIENTS[1]},1000\n${SAMPLE_RECIPIENTS[2]},1000`,
        revocable: '1',
        schedule: () => ({
            cliffTs: timestampDaysFromNow(7),
            endTs: timestampDaysFromNow(90),
            kind: 'CliffLinear',
            startTs: timestampDaysFromNow(0),
        }),
    },
] as const;

export function CreateMerkleDropForm() {
    const { createMerkleDistribution } = useRewardsMutations();
    const { defaultMint, rememberDistribution, rememberMint } = useSavedValues();
    const { clusterMint, mint, setMint, setTokenProgram, tokenProgram } = useTokenFormDefaults();
    const [revocable, setRevocable] = useState('0');
    const [amount, setAmount] = useState('');
    const [clawbackTs, setClawbackTs] = useState('0');
    const [recipients, setRecipients] = useState('');
    const [schedule, setSchedule] = useState<VestingScheduleState>(INITIAL_VESTING_SCHEDULE);
    const [builtBundle, setBuiltBundle] = useState<BuiltProofDropBundle | null>(null);
    const [generatedSeed, setGeneratedSeed] = useState('');
    const [generatedDistribution, setGeneratedDistribution] = useState('');
    const [generatedBundleText, setGeneratedBundleText] = useState('');
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        createMerkleDistribution.reset();
        setFormError(null);
        setGeneratedBundleText('');

        const recipientResult = parseProofDropRecipients(recipients, schedule);
        if (!recipientResult.ok) {
            setFormError(recipientResult.error);
            return;
        }

        const bundleResult = buildProofDropBundle({ mint, recipients: recipientResult.value });
        if (!bundleResult.ok) {
            setFormError(bundleResult.error);
            return;
        }

        const validationError = firstValidationError(
            validateAddress(mint, 'Mint address'),
            validatePositiveInteger(amount, 'Initial funded amount'),
            validateInteger(clawbackTs, 'Clawback timestamp'),
            validateOptionalAddress(tokenProgram, 'Token program'),
        );
        if (validationError) {
            setFormError(validationError);
            return;
        }

        const result = await createMerkleDistribution
            .mutateAsync({
                amount: parseBigIntValue(amount),
                clawbackTs: parseBigIntValue(clawbackTs),
                merkleRoot: bundleResult.value.merkleRoot,
                mint,
                revocable: Number(revocable),
                tokenProgram,
                totalAmount: bundleResult.value.totalAmount,
            })
            .catch(() => null);
        if (!result) return;

        const bundleWithDistribution = buildProofDropBundle({
            distribution: result.distribution,
            mint: result.mint,
            recipients: recipientResult.value,
        });
        setGeneratedSeed(result.seed);
        setGeneratedDistribution(result.distribution);
        setBuiltBundle(bundleWithDistribution.ok ? bundleWithDistribution.value : bundleResult.value);
        setGeneratedBundleText(
            proofDropBundleText(
                bundleWithDistribution.ok ? bundleWithDistribution.value.bundle : bundleResult.value.bundle,
            ),
        );
        rememberDistribution(result.distribution);
        rememberMint(result.mint);
    };

    const handlePreview = () => {
        setFormError(null);
        setGeneratedBundleText('');

        const recipientResult = parseProofDropRecipients(recipients, schedule);
        if (!recipientResult.ok) {
            setBuiltBundle(null);
            setFormError(recipientResult.error);
            return;
        }

        const bundleResult = buildProofDropBundle({ mint, recipients: recipientResult.value });
        if (!bundleResult.ok) {
            setBuiltBundle(null);
            setFormError(bundleResult.error);
            return;
        }

        setBuiltBundle(bundleResult.value);
    };

    const applyTemplate = (template: ProofDropTemplate) => {
        setAmount(template.amount);
        setClawbackTs(timestampDaysFromNow(template.clawbackDays));
        setGeneratedBundleText('');
        setGeneratedDistribution('');
        setGeneratedSeed('');
        setRecipients(template.recipients);
        setRevocable(template.revocable);
        setSchedule(template.schedule());
        setBuiltBundle(null);
        setFormError(null);
    };

    return (
        <form onSubmit={e => void handleSubmit(e)} className="flex flex-col gap-4">
            <div className="flex flex-wrap gap-2">
                {PROOF_DROP_TEMPLATES.map(template => (
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
                label="Mint Address"
                value={mint}
                onChange={setMint}
                autoFillValue={defaultMint || clusterMint}
                onAutoFill={setMint}
                placeholder="SPL token mint"
                required
            />
            <SelectField
                label="Revocable"
                value={revocable}
                onChange={setRevocable}
                options={[
                    { label: 'No (0)', value: '0' },
                    { label: 'Yes (1)', value: '1' },
                ]}
            />
            <FormField
                label="Initial Funded Amount"
                value={amount}
                onChange={setAmount}
                type="number"
                placeholder="Amount transferred into vault on creation"
                required
            />
            <FormField
                label="Clawback Timestamp (i64)"
                value={clawbackTs}
                onChange={setClawbackTs}
                type="number"
                placeholder="Unix timestamp"
                required
            />
            <TextAreaField
                label="Recipient Allocations"
                value={recipients}
                onChange={setRecipients}
                placeholder={`recipient,amount\n7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgS5U,1000`}
                hint="CSV lines or JSON recipients array"
                required
                rows={6}
            />
            <VestingScheduleField value={schedule} onChange={setSchedule} />
            <Button type="button" variant="secondary" size="sm" onClick={handlePreview} className="self-start">
                Generate Preview
            </Button>
            {builtBundle && (
                <div className="grid gap-3 rounded-lg border bg-background p-3 text-sm">
                    <div className="grid gap-1">
                        <span className="text-xs text-muted-foreground">Recipients</span>
                        <span className="font-semibold">{builtBundle.recipientCount}</span>
                    </div>
                    <div className="grid gap-1">
                        <span className="text-xs text-muted-foreground">Total Allocation</span>
                        <span className="font-semibold">{builtBundle.totalAmount.toString()}</span>
                    </div>
                    <div className="grid gap-1">
                        <span className="text-xs text-muted-foreground">Generated Root</span>
                        <span className="break-all font-mono text-xs">0x{builtBundle.merkleRootHex}</span>
                    </div>
                </div>
            )}
            <FormField
                label="Token Program"
                value={tokenProgram}
                onChange={setTokenProgram}
                placeholder="Token program address"
            />
            {generatedSeed && (
                <FormField
                    label="Generated Seed"
                    value={generatedSeed}
                    onChange={() => {}}
                    readOnly
                    hint="Random signer used to derive drop address"
                />
            )}
            {generatedDistribution && (
                <FormField label="Generated Drop Address" value={generatedDistribution} onChange={() => {}} readOnly />
            )}
            {generatedBundleText && (
                <TextAreaField
                    label="Recipient Proof Bundle"
                    value={generatedBundleText}
                    onChange={() => {}}
                    readOnly
                    rows={10}
                />
            )}
            <SendButton sending={createMerkleDistribution.isPending} />
            <TxResult
                signature={createMerkleDistribution.data?.signature}
                error={formError ?? createMerkleDistribution.error}
            />
        </form>
    );
}
