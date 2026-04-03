'use client';

import { Button } from '@solana/design-system/button';
import { Select, SelectItem } from '@solana/design-system/select';
import { TextInput } from '@solana/design-system/text-input';
import { BalanceSource, RevokeMode, type VestingScheduleArgs } from '@solana/rewards-client';
import { parseBigIntValue, validateInteger } from '@/lib/validation';

interface FormFieldProps {
    label: string;
    value: string;
    onChange: (v: string) => void;
    placeholder?: string;
    hint?: string;
    required?: boolean;
    readOnly?: boolean;
    type?: string;
    autoFillValue?: string;
    onAutoFill?: (v: string) => void;
    autoFillLabel?: string;
}

export function FormField({
    label,
    value,
    onChange,
    placeholder,
    hint,
    required,
    readOnly,
    type = 'text',
    autoFillValue = '',
    onAutoFill,
    autoFillLabel = 'Autofill',
}: FormFieldProps) {
    return (
        <TextInput
            label={label}
            description={hint}
            action={
                onAutoFill ? (
                    <Button
                        type="button"
                        size="sm"
                        variant="secondary"
                        onClick={() => onAutoFill(autoFillValue)}
                        disabled={!autoFillValue}
                    >
                        {autoFillLabel}
                    </Button>
                ) : undefined
            }
            inputClassName={readOnly ? 'opacity-60' : undefined}
            type={type}
            value={value}
            onChange={e => onChange(e.target.value)}
            placeholder={placeholder}
            required={required}
            readOnly={readOnly}
        />
    );
}

interface TextAreaFieldProps {
    label: string;
    value: string;
    onChange: (v: string) => void;
    placeholder?: string;
    hint?: string;
    required?: boolean;
}

export function TextAreaField({ label, value, onChange, placeholder, hint, required }: TextAreaFieldProps) {
    return (
        <label style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
            <span style={{ fontSize: '0.8125rem', fontWeight: 500 }}>{label}</span>
            {hint && <span style={{ fontSize: '0.75rem', color: 'var(--color-muted)' }}>{hint}</span>}
            <textarea
                value={value}
                onChange={e => onChange(e.target.value)}
                placeholder={placeholder}
                required={required}
                rows={4}
                style={{
                    background: 'var(--color-bg)',
                    border: '1px solid var(--color-border)',
                    color: 'var(--color-text)',
                    borderRadius: 8,
                    padding: '10px 12px',
                    fontSize: '0.8125rem',
                    resize: 'vertical',
                }}
            />
        </label>
    );
}

interface SelectFieldProps {
    label: string;
    value: string;
    onChange: (v: string) => void;
    options: { label: string; value: string }[];
    hint?: string;
}

export function SelectField({ label, value, onChange, options, hint }: SelectFieldProps) {
    return (
        <Select label={label} description={hint} value={value} onValueChange={nextValue => onChange(nextValue ?? '')}>
            {options.map(option => (
                <SelectItem key={option.value} value={option.value}>
                    {option.label}
                </SelectItem>
            ))}
        </Select>
    );
}

export function SendButton({ sending }: { sending: boolean }) {
    return (
        <Button type="submit" loading={sending} disabled={sending} style={{ marginTop: 8, alignSelf: 'flex-start' }}>
            {sending ? 'Sending Transaction' : 'Send Transaction'}
        </Button>
    );
}

export interface VestingScheduleState {
    kind: 'Immediate' | 'Linear' | 'Cliff' | 'CliffLinear';
    startTs: string;
    cliffTs: string;
    endTs: string;
}

export const INITIAL_VESTING_SCHEDULE: VestingScheduleState = {
    kind: 'Immediate',
    startTs: '',
    cliffTs: '',
    endTs: '',
};

export function VestingScheduleField({
    value,
    onChange,
}: {
    value: VestingScheduleState;
    onChange: (next: VestingScheduleState) => void;
}) {
    return (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            <SelectField
                label="Vesting Schedule"
                value={value.kind}
                onChange={next => onChange({ ...value, kind: next as VestingScheduleState['kind'] })}
                options={[
                    { label: 'Immediate', value: 'Immediate' },
                    { label: 'Linear', value: 'Linear' },
                    { label: 'Cliff', value: 'Cliff' },
                    { label: 'CliffLinear', value: 'CliffLinear' },
                ]}
            />
            {(value.kind === 'Linear' || value.kind === 'CliffLinear') && (
                <FormField
                    label="Start Timestamp (i64)"
                    value={value.startTs}
                    onChange={startTs => onChange({ ...value, startTs })}
                    placeholder="Unix timestamp"
                    type="number"
                    required
                />
            )}
            {(value.kind === 'Cliff' || value.kind === 'CliffLinear') && (
                <FormField
                    label="Cliff Timestamp (i64)"
                    value={value.cliffTs}
                    onChange={cliffTs => onChange({ ...value, cliffTs })}
                    placeholder="Unix timestamp"
                    type="number"
                    required
                />
            )}
            {(value.kind === 'Linear' || value.kind === 'CliffLinear') && (
                <FormField
                    label="End Timestamp (i64)"
                    value={value.endTs}
                    onChange={endTs => onChange({ ...value, endTs })}
                    placeholder="Unix timestamp"
                    type="number"
                    required
                />
            )}
        </div>
    );
}

export function buildVestingSchedule(
    value: VestingScheduleState,
): { ok: true; value: VestingScheduleArgs } | { ok: false; error: string } {
    if (value.kind === 'Immediate') {
        return { ok: true, value: { __kind: 'Immediate' } };
    }

    if (value.kind === 'Linear') {
        const startErr = validateInteger(value.startTs, 'Start timestamp');
        if (startErr) return { ok: false, error: startErr };
        const endErr = validateInteger(value.endTs, 'End timestamp');
        if (endErr) return { ok: false, error: endErr };
        return {
            ok: true,
            value: {
                __kind: 'Linear',
                startTs: parseBigIntValue(value.startTs),
                endTs: parseBigIntValue(value.endTs),
            },
        };
    }

    if (value.kind === 'Cliff') {
        const cliffErr = validateInteger(value.cliffTs, 'Cliff timestamp');
        if (cliffErr) return { ok: false, error: cliffErr };
        return {
            ok: true,
            value: {
                __kind: 'Cliff',
                cliffTs: parseBigIntValue(value.cliffTs),
            },
        };
    }

    const startErr = validateInteger(value.startTs, 'Start timestamp');
    if (startErr) return { ok: false, error: startErr };
    const cliffErr = validateInteger(value.cliffTs, 'Cliff timestamp');
    if (cliffErr) return { ok: false, error: cliffErr };
    const endErr = validateInteger(value.endTs, 'End timestamp');
    if (endErr) return { ok: false, error: endErr };

    return {
        ok: true,
        value: {
            __kind: 'CliffLinear',
            startTs: parseBigIntValue(value.startTs),
            cliffTs: parseBigIntValue(value.cliffTs),
            endTs: parseBigIntValue(value.endTs),
        },
    };
}

export function RevokeModeField({ value, onChange }: { value: RevokeMode; onChange: (value: RevokeMode) => void }) {
    return (
        <SelectField
            label="Revoke Mode"
            value={String(value)}
            onChange={next => onChange(Number(next) as RevokeMode)}
            options={[
                { label: 'NonVested (0)', value: String(RevokeMode.NonVested) },
                { label: 'Full (1)', value: String(RevokeMode.Full) },
            ]}
            hint="NonVested sends vested rewards to user; Full returns all to authority"
        />
    );
}

export function BalanceSourceField({
    value,
    onChange,
}: {
    value: BalanceSource;
    onChange: (value: BalanceSource) => void;
}) {
    return (
        <SelectField
            label="Balance Source"
            value={String(value)}
            onChange={next => onChange(Number(next) as BalanceSource)}
            options={[
                { label: 'OnChain (0)', value: String(BalanceSource.OnChain) },
                { label: 'AuthoritySet (1)', value: String(BalanceSource.AuthoritySet) },
            ]}
            hint="OnChain reads user token balance on-chain; AuthoritySet accepts authority-updated balances"
        />
    );
}
