'use client';

import type { Address } from '@solana/kit';

import { deriveEventAuthority, normalizeTokenProgram } from '@/lib/pdas';
import { getProgramAddress } from '@/lib/program';

export function getProgramConfig() {
    const programAddress = getProgramAddress();
    const eventAuthority = deriveEventAuthority(programAddress);
    return {
        eventAuthority,
        programAddress,
    } as const;
}

export function asAddress(value: string): Address {
    return value.trim() as Address;
}

export function normalizeTokenProgramInput(value: string): Address {
    return normalizeTokenProgram(value);
}
