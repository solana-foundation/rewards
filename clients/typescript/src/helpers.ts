/**
 * Parses a user-provided integer string into a bigint.
 */
export function parseBigIntValue(value: string): bigint {
    return BigInt(value.trim());
}

function isByte(value: unknown): value is number {
    if (typeof value !== 'number') return false;
    return Number.isInteger(value) && value >= 0 && value <= 255;
}

function isByteArray32(value: unknown): value is number[] {
    return Array.isArray(value) && value.length === 32 && value.every(isByte);
}

/**
 * Parses a 32-byte value from either a hex string or JSON array.
 */
export function parseByteArray32(
    value: string,
    label: string,
): { error: string; ok: false } | { ok: true; value: number[] } {
    const normalized = value.trim();
    if (!normalized) return { error: `${label} is required.`, ok: false };

    const hex = normalized.startsWith('0x') ? normalized.slice(2) : normalized;
    if (/^[0-9a-fA-F]{64}$/.test(hex)) {
        const bytes = Array.from({ length: 32 }, (_, i) => Number.parseInt(hex.slice(i * 2, i * 2 + 2), 16));
        return { ok: true, value: bytes };
    }

    try {
        const parsed: unknown = JSON.parse(normalized);
        if (isByteArray32(parsed)) {
            return { ok: true, value: parsed };
        }
    } catch {
        // noop
    }

    return { error: `${label} must be 32 bytes (hex string or JSON number array).`, ok: false };
}

/**
 * Parses a Merkle proof from JSON or newline-delimited nodes.
 */
export function parseMerkleProof(value: string): { error: string; ok: false } | { ok: true; value: number[][] } {
    const normalized = value.trim();
    if (!normalized) return { ok: true, value: [] };

    try {
        const parsed: unknown = JSON.parse(normalized);
        if (Array.isArray(parsed) && parsed.every(node => isByteArray32(node))) {
            return { ok: true, value: parsed };
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
        if (!parsed.ok) return { error: parsed.error, ok: false };
        nodes.push(parsed.value);
    }

    return { ok: true, value: nodes };
}
