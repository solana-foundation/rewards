import { useState } from 'react';
import { useWallet } from '@solana/connector/react';
import { Button } from '@solana/design-system';

import { useSavedValues } from '@/contexts/SavedValuesContext';
import type { ProofDropClaimDraft } from '@/hooks/use-proof-drop-claims';
import { parseProofDropClaimBundle } from '@/lib/proof-drop-bundle';
import { TextAreaField } from '../shared/reward-form-fields';

interface ImportProofDropFormProps {
    onImport: (claim: ProofDropClaimDraft) => void;
}

export function ImportProofDropForm({ onImport }: ImportProofDropFormProps) {
    const { account } = useWallet();
    const { rememberDistribution } = useSavedValues();
    const [bundle, setBundle] = useState('');
    const [formError, setFormError] = useState<string | null>(null);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        setFormError(null);

        if (!account) {
            setFormError('Connect wallet to add a proof-based drop.');
            return;
        }

        const claimResult = parseProofDropClaimBundle(bundle, { claimant: account, requireDistribution: true });
        if (!claimResult.ok) {
            setFormError(claimResult.error);
            return;
        }

        rememberDistribution(claimResult.value.distribution);
        onImport({
            distribution: claimResult.value.distribution,
            proof: claimResult.value.proofText,
            schedule: claimResult.value.schedule,
            totalAmount: claimResult.value.totalAmount,
        });
    };

    return (
        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
            <TextAreaField
                label="Recipient Proof Bundle"
                value={bundle}
                onChange={setBundle}
                placeholder='{"kind":"proof-drop","distribution":"...","recipients":[...]}'
                rows={10}
                required
            />
            {formError && <p className="text-sm text-destructive">{formError}</p>}
            <Button type="submit" className="mt-2 self-start">
                Add to My Rewards
            </Button>
        </form>
    );
}
