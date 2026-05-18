import { useState } from 'react';
import { Button } from '@solana/design-system';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useRewardsMutations } from '@/hooks/use-rewards-mutations';
import {
    firstValidationError,
    validateAddress,
    validateInteger,
    validateOptionalAddress,
    parseBigIntValue,
} from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, SelectField, SendButton } from '../shared/reward-form-fields';

const DAY_SECONDS = 86_400;

const CAMPAIGN_TEMPLATES = [
    { clawbackDays: 0, label: 'Open', revocable: '0' },
    { clawbackDays: 30, label: 'Revocable 30d', revocable: '1' },
    { clawbackDays: 90, label: 'Close 90d', revocable: '0' },
] as const;

function timestampDaysFromNow(days: number) {
    if (days === 0) return '0';
    return String(Math.floor(Date.now() / 1000) + days * DAY_SECONDS);
}

export function CreateDirectRewardForm() {
    const { createDirectDistribution } = useRewardsMutations();
    const { defaultMint, rememberDistribution, rememberMint } = useSavedValues();
    const [mint, setMint] = useState('');
    const [revocable, setRevocable] = useState('0');
    const [clawbackTs, setClawbackTs] = useState('0');
    const [tokenProgram, setTokenProgram] = useState('');
    const [generatedSeed, setGeneratedSeed] = useState('');
    const [generatedDistribution, setGeneratedDistribution] = useState('');
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        createDirectDistribution.reset();
        setFormError(null);

        const validationError = firstValidationError(
            validateAddress(mint, 'Mint address'),
            validateInteger(clawbackTs, 'Clawback timestamp'),
            validateOptionalAddress(tokenProgram, 'Token program'),
        );
        if (validationError) {
            setFormError(validationError);
            return;
        }

        const result = await createDirectDistribution
            .mutateAsync({
                clawbackTs: parseBigIntValue(clawbackTs),
                mint,
                revocable: Number(revocable),
                tokenProgram,
            })
            .catch(() => null);
        if (!result) return;

        setGeneratedSeed(result.seed);
        setGeneratedDistribution(result.distribution);
        rememberDistribution(result.distribution);
        rememberMint(result.mint);
    };

    const applyTemplate = (template: (typeof CAMPAIGN_TEMPLATES)[number]) => {
        setRevocable(template.revocable);
        setClawbackTs(timestampDaysFromNow(template.clawbackDays));
    };

    return (
        <form onSubmit={e => void handleSubmit(e)} className="flex flex-col gap-4">
            <div className="flex flex-wrap gap-2">
                {CAMPAIGN_TEMPLATES.map(template => (
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
                autoFillValue={defaultMint}
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
                label="Clawback Timestamp (i64)"
                value={clawbackTs}
                onChange={setClawbackTs}
                type="number"
                placeholder="Unix timestamp"
                hint="Authority can close reward after this timestamp"
                required
            />
            <FormField
                label="Token Program"
                value={tokenProgram}
                onChange={setTokenProgram}
                placeholder="Defaults to Tokenkeg..."
                hint="Use Token-2022 program address for Token-2022 mints"
            />
            {generatedSeed && (
                <FormField
                    label="Generated Seed"
                    value={generatedSeed}
                    onChange={() => {}}
                    readOnly
                    hint="Random signer used to derive reward address"
                />
            )}
            {generatedDistribution && (
                <FormField
                    label="Generated Reward Address"
                    value={generatedDistribution}
                    onChange={() => {}}
                    readOnly
                    hint="Saved as default reward after success"
                />
            )}
            <SendButton sending={createDirectDistribution.isPending} />
            <TxResult
                signature={createDirectDistribution.data?.signature}
                error={formError ?? createDirectDistribution.error}
            />
        </form>
    );
}
