import fs from 'fs';
import path from 'path';

interface ConfigPreserver {
    restore: () => void;
}

const TYPESCRIPT_CLIENT_HELPERS_FILENAME = 'helpers.ts';
const TYPESCRIPT_CLIENT_INDEX_FILENAME = 'index.ts';
const TYPESCRIPT_CLIENT_HELPERS_EXPORT_LINE = 'export * from "./helpers";';
const TYPESCRIPT_CLIENT_HELPERS_SOURCE = `/**
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
): { ok: true; value: number[] } | { ok: false; error: string } {
    const normalized = value.trim();
    if (!normalized) return { ok: false, error: \`\${label} is required.\` };

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

    return { ok: false, error: \`\${label} must be 32 bytes (hex string or JSON number array).\` };
}

/**
 * Parses a Merkle proof from JSON or newline-delimited nodes.
 */
export function parseMerkleProof(value: string): { ok: true; value: number[][] } | { ok: false; error: string } {
    const normalized = value.trim();
    if (!normalized) return { ok: true, value: [] };

    try {
        const parsed: unknown = JSON.parse(normalized);
        if (Array.isArray(parsed) && parsed.every(node => isByteArray32(node))) {
            return { ok: true, value: parsed as number[][] };
        }
    } catch {
        // noop
    }

    const lines = normalized
        .split(/\\r?\\n/)
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
`;

/**
 * Preserves config files (Cargo.toml, package.json, etc.) during client generation.
 */
export function preserveConfigFiles(typescriptClientsDir: string, rustClientsDir?: string): ConfigPreserver {
    const filesToPreserve = ['package.json', 'tsconfig.json', '.npmignore', 'pnpm-lock.yaml', 'Cargo.toml'];
    const preservedFiles = new Map<string, string>();

    filesToPreserve.forEach(filename => {
        const filePath = path.join(typescriptClientsDir, filename);
        const tempPath = path.join(typescriptClientsDir, `${filename}.temp`);

        if (fs.existsSync(filePath)) {
            fs.copyFileSync(filePath, tempPath);
            preservedFiles.set(filename, tempPath);
        }
    });

    const rustCargoPath = rustClientsDir ? path.join(rustClientsDir, 'Cargo.toml') : null;
    const rustCargoTempPath = rustClientsDir ? path.join(rustClientsDir, 'Cargo.toml.temp') : null;

    if (rustCargoPath && rustCargoTempPath && fs.existsSync(rustCargoPath)) {
        fs.copyFileSync(rustCargoPath, rustCargoTempPath);
        preservedFiles.set('rust_cargo', rustCargoTempPath);
    }

    return {
        restore: () => {
            preservedFiles.forEach((tempPath, filename) => {
                try {
                    if (filename === 'rust_cargo') {
                        const filePath = path.join(rustClientsDir!, 'Cargo.toml');
                        if (fs.existsSync(tempPath)) {
                            fs.copyFileSync(tempPath, filePath);
                            fs.unlinkSync(tempPath);
                        }
                    } else {
                        const filePath = path.join(typescriptClientsDir, filename);
                        if (fs.existsSync(tempPath)) {
                            fs.copyFileSync(tempPath, filePath);
                            fs.unlinkSync(tempPath);
                        }
                    }
                } catch (error) {
                    // Silently handle cleanup errors - they shouldn't fail the build
                    console.warn(`Warning: Failed to cleanup temporary file ${tempPath}:`, (error as Error).message);
                }
            });
        },
    };
}

/**
 * Adds manually maintained helpers to the generated TypeScript client and exports them.
 */
export function addTypescriptClientHelpers(typescriptClientsDir: string): void {
    const generatedDir = path.join(typescriptClientsDir, 'src', 'generated');
    const helpersPath = path.join(generatedDir, TYPESCRIPT_CLIENT_HELPERS_FILENAME);
    const indexPath = path.join(generatedDir, TYPESCRIPT_CLIENT_INDEX_FILENAME);

    fs.writeFileSync(helpersPath, TYPESCRIPT_CLIENT_HELPERS_SOURCE);

    if (!fs.existsSync(indexPath)) {
        throw new Error(`Could not find generated client index at ${indexPath}`);
    }

    const indexContent = fs.readFileSync(indexPath, 'utf8');
    if (indexContent.includes(TYPESCRIPT_CLIENT_HELPERS_EXPORT_LINE)) {
        return;
    }

    fs.writeFileSync(indexPath, `${indexContent.trimEnd()}\n${TYPESCRIPT_CLIENT_HELPERS_EXPORT_LINE}\n`);
}
