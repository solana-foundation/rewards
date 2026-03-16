'use client';

import { PublicKey } from '@solana/web3.js';

export { parseBigIntValue, parseByteArray32, parseMerkleProof } from '@solana/rewards-client';

function normalize(value: string) {
    return value.trim();
}

export function validateAddress(value: string, label: string): string | null {
    const normalized = normalize(value);
    if (!normalized) return `${label} is required.`;
    try {
        void new PublicKey(normalized);
        return null;
    } catch {
        return `${label} is not a valid Solana address.`;
    }
}

export function validateOptionalAddress(value: string, label: string): string | null {
    const normalized = normalize(value);
    if (!normalized) return null;
    return validateAddress(normalized, label);
}

export function validateInteger(value: string, label: string): string | null {
    const normalized = normalize(value);
    if (!normalized) return `${label} is required.`;
    if (!/^-?\d+$/.test(normalized)) return `${label} must be an integer.`;
    return null;
}

export function validatePositiveInteger(value: string, label: string): string | null {
    const normalized = normalize(value);
    if (!normalized) return `${label} is required.`;
    if (!/^\d+$/.test(normalized)) return `${label} must be a whole number.`;
    try {
        const parsed = BigInt(normalized);
        if (parsed <= 0n) return `${label} must be greater than 0.`;
        return null;
    } catch {
        return `${label} is not a valid integer value.`;
    }
}

export function validateNonNegativeInteger(value: string, label: string): string | null {
    const normalized = normalize(value);
    if (!normalized) return `${label} is required.`;
    if (!/^\d+$/.test(normalized)) return `${label} must be a whole number.`;
    return null;
}

export function firstValidationError(...errors: Array<string | null>): string | null {
    return errors.find(Boolean) ?? null;
}
