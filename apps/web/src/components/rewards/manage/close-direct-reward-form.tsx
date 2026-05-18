import { useState } from 'react';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useRewardsMutations } from '@/hooks/use-rewards-mutations';
import { firstValidationError, validateAddress, validateOptionalAddress } from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, SendButton } from '../shared/reward-form-fields';

interface CloseDirectRewardFormProps {
    hideKnownFields?: boolean;
    initialDistribution?: string;
    initialMint?: string;
    onSuccess?: () => void;
    submitLabel?: string;
}

export function CloseDirectRewardForm({
    hideKnownFields = false,
    initialDistribution = '',
    initialMint = '',
    onSuccess,
    submitLabel,
}: CloseDirectRewardFormProps) {
    const { closeDirectDistribution } = useRewardsMutations();
    const { defaultDistribution, defaultMint, rememberDistribution, rememberMint } = useSavedValues();
    const [distribution, setDistribution] = useState(initialDistribution);
    const [mint, setMint] = useState(initialMint);
    const [tokenProgram, setTokenProgram] = useState('');
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        closeDirectDistribution.reset();
        setFormError(null);

        const validationError = firstValidationError(
            validateAddress(distribution, 'Reward address'),
            validateAddress(mint, 'Mint address'),
            validateOptionalAddress(tokenProgram, 'Token program'),
        );
        if (validationError) {
            setFormError(validationError);
            return;
        }

        const result = await closeDirectDistribution
            .mutateAsync({
                distribution,
                mint,
                tokenProgram,
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
                        autoFillValue={defaultMint}
                        onAutoFill={setMint}
                        placeholder="SPL token mint"
                        required
                    />
                </>
            )}
            <FormField
                label="Token Program"
                value={tokenProgram}
                onChange={setTokenProgram}
                placeholder="Defaults to Tokenkeg..."
            />
            <SendButton sending={closeDirectDistribution.isPending} label={submitLabel} />
            <TxResult
                signature={closeDirectDistribution.data?.signature}
                error={formError ?? closeDirectDistribution.error}
            />
        </form>
    );
}
