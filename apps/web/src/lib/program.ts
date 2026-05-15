'use client';

import type { Address } from '@solana/kit';
import { REWARDS_PROGRAM_PROGRAM_ADDRESS } from '@solana/rewards';
import { PublicKey } from '@solana/web3.js';

export const SYSTEM_PROGRAM_ID = '11111111111111111111111111111111' as Address;
export const TOKEN_PROGRAM_ID = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA' as Address;
export const TOKEN_2022_PROGRAM_ID = 'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb' as Address;
export const ASSOCIATED_TOKEN_PROGRAM_ID = 'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL' as Address;
export const PROGRAM_ID_STORAGE_KEY = 'rewards-program-id';

export function getDefaultProgramAddress(): Address {
    return (process.env.NEXT_PUBLIC_PROGRAM_ID ?? REWARDS_PROGRAM_PROGRAM_ADDRESS) as Address;
}

function isValidProgramAddress(value: string) {
    try {
        void new PublicKey(value);
        return true;
    } catch {
        return false;
    }
}

export function getStoredProgramAddress(): Address | null {
    if (typeof window === 'undefined') return null;
    const storedValue = window.localStorage.getItem(PROGRAM_ID_STORAGE_KEY)?.trim();
    if (!storedValue || !isValidProgramAddress(storedValue)) return null;
    return storedValue as Address;
}

export function setStoredProgramAddress(value: string): Address | null {
    const normalized = value.trim();
    if (!normalized || !isValidProgramAddress(normalized)) return null;
    window.localStorage.setItem(PROGRAM_ID_STORAGE_KEY, normalized);
    return normalized as Address;
}

export function clearStoredProgramAddress() {
    if (typeof window === 'undefined') return;
    window.localStorage.removeItem(PROGRAM_ID_STORAGE_KEY);
}

export function getProgramAddress(): Address {
    return getStoredProgramAddress() ?? getDefaultProgramAddress();
}
