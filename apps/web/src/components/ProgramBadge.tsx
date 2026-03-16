'use client';

import { useEffect, useRef, useState } from 'react';

import { Button } from '@solana/design-system/button';
import { TextInput } from '@solana/design-system/text-input';

import {
    clearStoredProgramAddress,
    getDefaultProgramAddress,
    getProgramAddress,
    getStoredProgramAddress,
    setStoredProgramAddress,
} from '@/lib/program';
import { validateAddress } from '@/lib/validation';

function truncate(value: string, start = 4, end = 4) {
    if (value.length <= start + end + 3) return value;
    return `${value.slice(0, start)}...${value.slice(-end)}`;
}

export function ProgramBadge() {
    const [open, setOpen] = useState(false);
    const [customInput, setCustomInput] = useState('');
    const [error, setError] = useState<string | null>(null);
    const [programId, setProgramId] = useState(getDefaultProgramAddress());
    const [hasCustomProgramId, setHasCustomProgramId] = useState(false);
    const containerRef = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        setProgramId(getProgramAddress());
        setHasCustomProgramId(Boolean(getStoredProgramAddress()));
    }, []);

    useEffect(() => {
        const handlePointerDown = (event: MouseEvent) => {
            if (!open) return;
            if (!containerRef.current?.contains(event.target as Node)) {
                setOpen(false);
                setError(null);
            }
        };

        const handleEscape = (event: KeyboardEvent) => {
            if (event.key === 'Escape') {
                setOpen(false);
                setError(null);
            }
        };

        document.addEventListener('mousedown', handlePointerDown);
        document.addEventListener('keydown', handleEscape);
        return () => {
            document.removeEventListener('mousedown', handlePointerDown);
            document.removeEventListener('keydown', handleEscape);
        };
    }, [open]);

    const applyCustomProgramId = () => {
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
    };

    const resetToDefault = () => {
        clearStoredProgramAddress();
        setProgramId(getDefaultProgramAddress());
        setHasCustomProgramId(false);
        setCustomInput('');
        setError(null);
        setOpen(false);
    };

    const label = hasCustomProgramId ? `Custom ${truncate(programId)}` : 'Default Program';

    return (
        <div ref={containerRef} style={{ position: 'relative' }}>
            <Button
                onClick={() => setOpen(value => !value)}
                variant="secondary"
                size="sm"
                style={{
                    alignItems: 'center',
                    display: 'flex',
                    fontSize: '0.75rem',
                    gap: 4,
                }}
            >
                {label} ▾
            </Button>

            {open && (
                <div
                    style={{
                        background: 'var(--color-card)',
                        border: '1px solid var(--color-border)',
                        borderRadius: 6,
                        left: 0,
                        minWidth: 360,
                        overflow: 'hidden',
                        padding: 10,
                        position: 'absolute',
                        top: '110%',
                        zIndex: 100,
                    }}
                >
                    <div style={{ color: 'var(--color-muted)', fontSize: '0.75rem', marginBottom: 8 }}>
                        Active Program ID: {truncate(programId, 8, 8)}
                    </div>
                    <div style={{ marginBottom: 8 }}>
                        <TextInput
                            value={customInput}
                            onChange={e => setCustomInput(e.target.value)}
                            placeholder="Enter custom program ID"
                            size="md"
                            onKeyDown={e => {
                                if (e.key === 'Enter') {
                                    e.preventDefault();
                                    applyCustomProgramId();
                                }
                            }}
                        />
                    </div>
                    {error && (
                        <div style={{ color: 'var(--color-error)', fontSize: '0.75rem', marginBottom: 8 }}>{error}</div>
                    )}
                    <div style={{ alignItems: 'center', display: 'flex', gap: 8 }}>
                        <Button type="button" size="sm" onClick={applyCustomProgramId}>
                            Set Program ID
                        </Button>
                        <Button type="button" size="sm" variant="secondary" onClick={resetToDefault}>
                            Use Default
                        </Button>
                    </div>
                </div>
            )}
        </div>
    );
}
