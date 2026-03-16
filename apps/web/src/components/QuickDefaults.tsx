'use client';

import { Button } from '@solana/design-system/button';
import { TextInput } from '@solana/design-system/text-input';
import { useSavedValues } from '@/contexts/SavedValuesContext';

interface SavedFieldProps {
    label: string;
    value: string;
    onChange: (value: string) => void;
    onSave: (value: string) => void;
    savedValues: string[];
    datalistId: string;
    placeholder: string;
}

function SavedField({ label, value, onChange, onSave, savedValues, datalistId, placeholder }: SavedFieldProps) {
    return (
        <div>
            <TextInput
                label={label}
                description={`${savedValues.length} saved`}
                list={datalistId}
                value={value}
                onChange={e => onChange(e.target.value)}
                placeholder={placeholder}
                action={
                    <Button
                        type="button"
                        size="sm"
                        variant="secondary"
                        onClick={() => onSave(value)}
                        disabled={!value.trim()}
                    >
                        Save
                    </Button>
                }
            />
            <datalist id={datalistId}>
                {savedValues.map(savedValue => (
                    <option key={savedValue} value={savedValue} />
                ))}
            </datalist>
        </div>
    );
}

export function QuickDefaults() {
    const {
        defaultDistribution,
        defaultRewardPool,
        defaultMint,
        defaultTrackedMint,
        defaultRewardMint,
        distributions,
        rewardPools,
        mints,
        trackedMints,
        rewardMints,
        setDefaultDistribution,
        setDefaultRewardPool,
        setDefaultMint,
        setDefaultTrackedMint,
        setDefaultRewardMint,
        rememberDistribution,
        rememberRewardPool,
        rememberMint,
        rememberTrackedMint,
        rememberRewardMint,
        clearSavedValues,
    } = useSavedValues();

    return (
        <section
            style={{
                border: '1px solid var(--color-border)',
                borderRadius: 8,
                padding: 16,
                marginBottom: 24,
                background: 'var(--color-card)',
            }}
        >
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 12 }}>
                <h3 style={{ fontSize: '0.9375rem', fontWeight: 600 }}>Quick Defaults</h3>
                <Button type="button" size="sm" variant="secondary" onClick={clearSavedValues}>
                    Clear Saved
                </Button>
            </div>
            <div
                style={{
                    display: 'grid',
                    gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
                    gap: 12,
                }}
            >
                <SavedField
                    label="Default Distribution"
                    value={defaultDistribution}
                    onChange={setDefaultDistribution}
                    onSave={rememberDistribution}
                    savedValues={distributions}
                    datalistId="saved-distributions"
                    placeholder="Distribution PDA"
                />
                <SavedField
                    label="Default Reward Pool"
                    value={defaultRewardPool}
                    onChange={setDefaultRewardPool}
                    onSave={rememberRewardPool}
                    savedValues={rewardPools}
                    datalistId="saved-pools"
                    placeholder="Reward pool PDA"
                />
                <SavedField
                    label="Default Mint"
                    value={defaultMint}
                    onChange={setDefaultMint}
                    onSave={rememberMint}
                    savedValues={mints}
                    datalistId="saved-mints"
                    placeholder="SPL token mint"
                />
                <SavedField
                    label="Default Tracked Mint"
                    value={defaultTrackedMint}
                    onChange={setDefaultTrackedMint}
                    onSave={rememberTrackedMint}
                    savedValues={trackedMints}
                    datalistId="saved-tracked-mints"
                    placeholder="Tracked mint"
                />
                <SavedField
                    label="Default Reward Mint"
                    value={defaultRewardMint}
                    onChange={setDefaultRewardMint}
                    onSave={rememberRewardMint}
                    savedValues={rewardMints}
                    datalistId="saved-reward-mints"
                    placeholder="Reward mint"
                />
            </div>
        </section>
    );
}
