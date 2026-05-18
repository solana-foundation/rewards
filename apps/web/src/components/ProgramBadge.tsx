import { useEffect, useState } from 'react';
import { Button, TextInput } from '@solana/design-system';
import { ChevronDown, Code2 } from 'lucide-react';

import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
    clearStoredProgramAddress,
    getDefaultProgramAddress,
    getProgramAddress,
    getStoredProgramAddress,
    setStoredProgramAddress,
} from '@/lib/program';
import { ellipsify } from '@/lib/utils';
import { validateAddress } from '@/lib/validation';

export function ProgramBadge() {
    const [customInput, setCustomInput] = useState('');
    const [error, setError] = useState<string | null>(null);
    const [open, setOpen] = useState(false);
    const [programId, setProgramId] = useState(getDefaultProgramAddress());
    const [hasCustomProgramId, setHasCustomProgramId] = useState(false);

    useEffect(() => {
        setProgramId(getProgramAddress());
        setHasCustomProgramId(Boolean(getStoredProgramAddress()));
    }, []);

    function applyCustomProgramId() {
        const validationError = validateAddress(customInput, 'Program ID');
        if (validationError) {
            setError(validationError);
            return;
        }

        const nextProgramId = setStoredProgramAddress(customInput);
        if (!nextProgramId) {
            setError('Program ID is not a valid Solana address.');
            return;
        }

        setProgramId(nextProgramId);
        setHasCustomProgramId(true);
        setCustomInput('');
        setError(null);
        setOpen(false);
    }

    function resetToDefault() {
        clearStoredProgramAddress();
        setProgramId(getDefaultProgramAddress());
        setHasCustomProgramId(false);
        setCustomInput('');
        setError(null);
        setOpen(false);
    }

    const label = hasCustomProgramId ? `Custom ${ellipsify(programId, 4)}` : 'Default Program';

    return (
        <DropdownMenu open={open} onOpenChange={setOpen}>
            <DropdownMenuTrigger asChild>
                <Button
                    iconLeft={<Code2 />}
                    iconRight={<ChevronDown className="opacity-60" />}
                    size="sm"
                    variant="secondary"
                >
                    {label}
                </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-96 p-3">
                <DropdownMenuLabel className="space-y-1 px-0">
                    <div className="text-sm">Program ID</div>
                    <div className="font-mono text-xs text-muted-foreground">{ellipsify(programId, 8)}</div>
                </DropdownMenuLabel>
                <DropdownMenuSeparator />
                <div className="space-y-3">
                    <TextInput
                        value={customInput}
                        onChange={e => setCustomInput(e.target.value)}
                        placeholder="Enter custom program ID"
                        size="md"
                        inputClassName="font-mono"
                        onKeyDown={e => {
                            if (e.key === 'Enter') {
                                e.preventDefault();
                                applyCustomProgramId();
                            }
                        }}
                    />
                    {error && <div className="text-xs text-destructive">{error}</div>}
                    <div className="flex items-center gap-2">
                        <Button type="button" size="sm" onClick={applyCustomProgramId}>
                            Set Program ID
                        </Button>
                        <Button type="button" size="sm" variant="secondary" onClick={resetToDefault}>
                            Use Default
                        </Button>
                    </div>
                </div>
            </DropdownMenuContent>
        </DropdownMenu>
    );
}
