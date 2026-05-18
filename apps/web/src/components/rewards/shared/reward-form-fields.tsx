import { Button } from '@solana/design-system/button';
import { Select, SelectItem } from '@solana/design-system/select';
import { TextInput } from '@solana/design-system/text-input';
import { RevokeMode } from '@solana/rewards';
import { Label } from '@/components/ui/label';
import { buildVestingSchedule, INITIAL_VESTING_SCHEDULE, type VestingScheduleState } from '@/lib/vesting-schedule';

export { buildVestingSchedule, INITIAL_VESTING_SCHEDULE, type VestingScheduleState };

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
    readOnly?: boolean;
    required?: boolean;
    rows?: number;
}

export function TextAreaField({
    label,
    value,
    onChange,
    placeholder,
    hint,
    readOnly,
    required,
    rows = 4,
}: TextAreaFieldProps) {
    return (
        <label className="grid gap-1.5">
            <Label>{label}</Label>
            {hint && <span className="text-xs text-muted-foreground">{hint}</span>}
            <textarea
                value={value}
                onChange={e => onChange(e.target.value)}
                placeholder={placeholder}
                readOnly={readOnly}
                required={required}
                rows={rows}
                className="min-h-24 resize-y rounded-lg border bg-background px-3 py-2 text-sm text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring/50 read-only:opacity-70"
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

export function SendButton({ label = 'Send Transaction', sending }: { label?: string; sending: boolean }) {
    return (
        <Button type="submit" loading={sending} disabled={sending} className="mt-2 self-start">
            {sending ? 'Sending Transaction' : label}
        </Button>
    );
}

export function VestingScheduleField({
    value,
    onChange,
}: {
    value: VestingScheduleState;
    onChange: (next: VestingScheduleState) => void;
}) {
    return (
        <div className="flex flex-col gap-3">
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

export function RevokeModeField({ value, onChange }: { value: RevokeMode; onChange: (value: RevokeMode) => void }) {
    return (
        <SelectField
            label="Revoke Mode"
            value={String(value)}
            onChange={next => onChange(Number(next))}
            options={[
                { label: 'NonVested (0)', value: String(RevokeMode.NonVested) },
                { label: 'Full (1)', value: String(RevokeMode.Full) },
            ]}
            hint="NonVested sends vested rewards to user; Full returns all to authority"
        />
    );
}
