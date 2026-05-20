import { useState } from 'react';
import { useWallet } from '@solana/connector/react';
import { Badge } from '@solana/design-system/badge';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useRewardsMutations } from '@/hooks/use-rewards-mutations';
import { useTokenFormDefaults } from '@/hooks/use-token-form-defaults';
import {
    firstValidationError,
    parseBigIntValue,
    validateAddress,
    validateNonNegativeInteger,
    validateOptionalAddress,
} from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, SendButton } from '../shared/reward-form-fields';

interface ClaimDirectRewardFormProps {
    initialAmount?: string;
    initialDistribution?: string;
    initialMint?: string;
    onSuccess?: () => void;
    submitLabel?: string;
}

export function ClaimDirectRewardForm({
    initialAmount = '0',
    initialDistribution = '',
    initialMint = '',
    onSuccess,
    submitLabel,
}: ClaimDirectRewardFormProps) {
    const { account } = useWallet();
    const { claimDirect } = useRewardsMutations();
    const { defaultDistribution, defaultMint, rememberDistribution, rememberMint } = useSavedValues();
    const [distribution, setDistribution] = useState(initialDistribution);
    const { clusterMint, mint, setMint, setTokenProgram, tokenProgram } = useTokenFormDefaults(initialMint);
    const [amount, setAmount] = useState(initialAmount);
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        claimDirect.reset();
        setFormError(null);

        const validationError = firstValidationError(
            validateAddress(distribution, 'Reward address'),
            validateAddress(mint, 'Mint address'),
            validateNonNegativeInteger(amount, 'Amount'),
            validateOptionalAddress(tokenProgram, 'Token program'),
        );
        if (validationError) {
            setFormError(validationError);
            return;
        }

        const result = await claimDirect
            .mutateAsync({
                amount: parseBigIntValue(amount),
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
            <div>
                <Badge variant="info">Claiming wallet: {account ?? 'Not connected'}</Badge>
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
                label="Amount (0 for max claimable)"
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
            <SendButton sending={claimDirect.isPending} label={submitLabel} />
            <TxResult signature={claimDirect.data?.signature} error={formError ?? claimDirect.error} />
        </form>
    );
}
