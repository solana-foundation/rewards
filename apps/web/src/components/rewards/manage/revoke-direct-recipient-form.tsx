import { useState } from 'react';
import { RevokeMode } from '@solana/rewards';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useRewardsMutations } from '@/hooks/use-rewards-mutations';
import { useTokenFormDefaults } from '@/hooks/use-token-form-defaults';
import { firstValidationError, validateAddress, validateOptionalAddress } from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, RevokeModeField, SendButton } from '../shared/reward-form-fields';

interface RevokeDirectRecipientFormProps {
    hideKnownFields?: boolean;
    initialDistribution?: string;
    initialMint?: string;
    onSuccess?: () => void;
    submitLabel?: string;
}

export function RevokeDirectRecipientForm({
    hideKnownFields = false,
    initialDistribution = '',
    initialMint = '',
    onSuccess,
    submitLabel,
}: RevokeDirectRecipientFormProps) {
    const { revokeDirectRecipient } = useRewardsMutations();
    const { defaultDistribution, defaultMint, rememberDistribution, rememberMint } = useSavedValues();
    const [distribution, setDistribution] = useState(initialDistribution);
    const { clusterMint, mint, setMint, setTokenProgram, tokenProgram } = useTokenFormDefaults(initialMint);
    const [recipient, setRecipient] = useState('');
    const [originalPayer, setOriginalPayer] = useState('');
    const [revokeMode, setRevokeMode] = useState<RevokeMode>(RevokeMode.NonVested);
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        revokeDirectRecipient.reset();
        setFormError(null);

        const validationError = firstValidationError(
            validateAddress(distribution, 'Reward address'),
            validateAddress(mint, 'Mint address'),
            validateAddress(recipient, 'Recipient address'),
            validateAddress(originalPayer, 'Original payer'),
            validateOptionalAddress(tokenProgram, 'Token program'),
        );

        if (validationError) {
            setFormError(validationError);
            return;
        }

        const result = await revokeDirectRecipient
            .mutateAsync({
                distribution,
                mint,
                originalPayer,
                recipient,
                revokeMode,
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
                        autoFillValue={defaultMint || clusterMint}
                        onAutoFill={setMint}
                        placeholder="SPL token mint"
                        required
                    />
                </>
            )}
            <FormField
                label="Recipient Address"
                value={recipient}
                onChange={setRecipient}
                placeholder="Recipient wallet to revoke"
                required
            />
            <FormField
                label="Original Payer"
                value={originalPayer}
                onChange={setOriginalPayer}
                placeholder="Original payer receiving rent refund"
                required
            />
            <FormField
                label="Token Program"
                value={tokenProgram}
                onChange={setTokenProgram}
                placeholder="Token program address"
            />
            <RevokeModeField value={revokeMode} onChange={setRevokeMode} />
            <SendButton sending={revokeDirectRecipient.isPending} label={submitLabel} />
            <TxResult
                signature={revokeDirectRecipient.data?.signature}
                error={formError ?? revokeDirectRecipient.error}
            />
        </form>
    );
}
