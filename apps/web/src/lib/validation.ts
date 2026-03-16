'use client';

import { PublicKey } from '@solana/web3.js';

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

export function parseBigIntValue(value: string) {
    return BigInt(value.trim());
}

export function parseByteArray32(value: string, label: string): { ok: true; value: number[] } | { ok: false; error: string } {
    const normalized = value.trim();
    if (!normalized) return { ok: false, error: `${label} is required.` };

    const hex = normalized.startsWith('0x') ? normalized.slice(2) : normalized;
    if (/^[0-9a-fA-F]{64}$/.test(hex)) {
        const bytes = Array.from({ length: 32 }, (_, i) => Number.parseInt(hex.slice(i * 2, i * 2 + 2), 16));
        return { ok: true, value: bytes };
    }

    try {
        const parsed = JSON.parse(normalized);
        if (
            Array.isArray(parsed) &&
            parsed.length === 32 &&
            parsed.every(item => Number.isInteger(item) && item >= 0 && item <= 255)
        ) {
            return { ok: true, value: parsed as number[] };
        }
    } catch {
        // noop
    }

    return { ok: false, error: `${label} must be 32 bytes (hex string or JSON number array).` };
}

export function parseMerkleProof(value: string): { ok: true; value: number[][] } | { ok: false; error: string } {
    const normalized = value.trim();
    if (!normalized) return { ok: true, value: [] };

    try {
        const parsed = JSON.parse(normalized);
        if (
            Array.isArray(parsed) &&
            parsed.every(
                node =>
                    Array.isArray(node) &&
                    node.length === 32 &&
                    node.every(byte => Number.isInteger(byte) && byte >= 0 && byte <= 255),
            )
        ) {
            return { ok: true, value: parsed as number[][] };
        }
    } catch {
        // noop
    }

    const lines = normalized
        .split(/\r?\n/)
        .map(line => line.trim())
        .filter(Boolean);

    if (lines.length === 0) return { ok: true, value: [] };

    const nodes: number[][] = [];
    for (const line of lines) {
        const parsed = parseByteArray32(line, 'Merkle proof node');
        if (!parsed.ok) return { ok: false, error: parsed.error };
        nodes.push(parsed.value);
    }

    return { ok: true, value: nodes };
}

export function firstValidationError(...errors: Array<string | null>): string | null {
    return errors.find(Boolean) ?? null;
}
