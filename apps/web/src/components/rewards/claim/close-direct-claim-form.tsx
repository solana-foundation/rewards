import { useState } from 'react';
import { useWallet } from '@solana/connector/react';
import { Badge } from '@solana/design-system/badge';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useRewardsMutations } from '@/hooks/use-rewards-mutations';
import { firstValidationError, validateAddress } from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, SendButton } from '../shared/reward-form-fields';

interface CloseDirectClaimFormProps {
    onSuccess?: () => void;
}

export function CloseDirectClaimForm({ onSuccess }: CloseDirectClaimFormProps) {
    const { account } = useWallet();
    const { closeDirectRecipient } = useRewardsMutations();
    const { defaultDistribution } = useSavedValues();
    const [distribution, setDistribution] = useState('');
    const [originalPayer, setOriginalPayer] = useState('');
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        closeDirectRecipient.reset();
        setFormError(null);

        const validationError = firstValidationError(
            validateAddress(distribution, 'Reward address'),
            validateAddress(originalPayer, 'Original payer'),
        );
        if (validationError) {
            setFormError(validationError);
            return;
        }

        const result = await closeDirectRecipient
            .mutateAsync({
                distribution,
                originalPayer,
            })
            .catch(() => null);
        if (!result) return;

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
                label="Original Payer"
                value={originalPayer}
                onChange={setOriginalPayer}
                placeholder="Address receiving recipient-account rent refund"
                required
            />
            <SendButton sending={closeDirectRecipient.isPending} />
            <TxResult
                signature={closeDirectRecipient.data?.signature}
                error={formError ?? closeDirectRecipient.error}
            />
        </form>
    );
}
