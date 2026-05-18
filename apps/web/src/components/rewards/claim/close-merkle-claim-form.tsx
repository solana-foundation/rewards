import { useState } from 'react';
import { useWallet } from '@solana/connector/react';
import { Badge } from '@solana/design-system/badge';
import { useSavedValues } from '@/contexts/SavedValuesContext';
import { useRewardsMutations } from '@/hooks/use-rewards-mutations';
import { validateAddress } from '@/lib/validation';
import { TxResult } from '@/components/TxResult';
import { FormField, SendButton } from '../shared/reward-form-fields';

interface CloseMerkleClaimFormProps {
    onSuccess?: () => void;
}

export function CloseMerkleClaimForm({ onSuccess }: CloseMerkleClaimFormProps) {
    const { account } = useWallet();
    const { closeMerkleClaim } = useRewardsMutations();
    const { defaultDistribution } = useSavedValues();
    const [distribution, setDistribution] = useState('');
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        closeMerkleClaim.reset();
        setFormError(null);

        const validationError = validateAddress(distribution, 'Drop address');
        if (validationError) {
            setFormError(validationError);
            return;
        }

        const result = await closeMerkleClaim.mutateAsync({ distribution }).catch(() => null);
        if (!result) return;

        onSuccess?.();
    };

    return (
        <form onSubmit={e => void handleSubmit(e)} className="flex flex-col gap-4">
            <div>
                <Badge variant="info">Claiming wallet: {account ?? 'Not connected'}</Badge>
            </div>
            <FormField
                label="Drop Address"
                value={distribution}
                onChange={setDistribution}
                autoFillValue={defaultDistribution}
                onAutoFill={setDistribution}
                placeholder="Proof-based drop address"
                required
            />
            <SendButton sending={closeMerkleClaim.isPending} />
            <TxResult signature={closeMerkleClaim.data?.signature} error={formError ?? closeMerkleClaim.error} />
        </form>
    );
}
