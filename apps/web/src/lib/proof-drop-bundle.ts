import { keccak_256 } from '@noble/hashes/sha3.js';
import { getVestingScheduleEncoder, type VestingScheduleArgs } from '@solana/rewards';
import { PublicKey } from '@solana/web3.js';

import { parseBigIntValue, validateAddress, validatePositiveInteger } from '@/lib/validation';
import { buildVestingSchedule, isVestingScheduleState, type VestingScheduleState } from '@/lib/vesting-schedule';

type Result<T> = { ok: true; value: T } | { error: string; ok: false };

const LEAF_PREFIX = 0;
const MAX_U64 = 2n ** 64n - 1n;

export interface ProofDropRecipientDraft {
    readonly recipient: string;
    readonly schedule: VestingScheduleState;
    readonly totalAmount: string;
}

export interface ProofDropBundleRecipient {
    readonly proof: readonly (readonly number[])[];
    readonly recipient: string;
    readonly schedule: VestingScheduleState;
    readonly totalAmount: string;
}

export interface ProofDropBundle {
    readonly distribution?: string;
    readonly kind: 'proof-drop';
    readonly merkleRoot: readonly number[];
    readonly merkleRootHex: string;
    readonly mint?: string;
    readonly recipients: readonly ProofDropBundleRecipient[];
    readonly totalAmount: string;
    readonly version: 1;
}

export interface BuiltProofDropBundle {
    readonly bundle: ProofDropBundle;
    readonly merkleRoot: readonly number[];
    readonly merkleRootHex: string;
    readonly recipientCount: number;
    readonly totalAmount: bigint;
}

export interface ProofDropClaimDetails {
    readonly distribution: string;
    readonly proof: readonly (readonly number[])[];
    readonly proofText: string;
    readonly recipient: string;
    readonly schedule: VestingScheduleState;
    readonly totalAmount: string;
}

interface BuildProofDropBundleInput {
    readonly distribution?: string;
    readonly mint?: string;
    readonly recipients: readonly ProofDropRecipientDraft[];
}

interface ParseClaimBundleOptions {
    readonly claimant?: string;
    readonly distribution?: string;
    readonly requireDistribution?: boolean;
}

interface MerkleNode {
    readonly hash: Uint8Array;
    readonly leafIndexes: readonly number[];
}

const vestingScheduleEncoder = getVestingScheduleEncoder();

function normalize(value: string) {
    return value.trim();
}

function isRecord(value: unknown): value is Record<string, unknown> {
    return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

function isByte(value: unknown): value is number {
    return typeof value === 'number' && Number.isInteger(value) && value >= 0 && value <= 255;
}

function isByteArray32(value: unknown): value is number[] {
    return Array.isArray(value) && value.length === 32 && value.every(isByte);
}

function isMerkleProof(value: unknown): value is number[][] {
    return Array.isArray(value) && value.every(isByteArray32);
}

function bytesToHex(bytes: readonly number[] | Uint8Array) {
    return Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('');
}

function bytesToArray(bytes: Uint8Array): number[] {
    return Array.from(bytes);
}

function concatBytes(...arrays: readonly Uint8Array[]) {
    const size = arrays.reduce((sum, array) => sum + array.length, 0);
    const result = new Uint8Array(size);
    let offset = 0;
    for (const array of arrays) {
        result.set(array, offset);
        offset += array.length;
    }
    return result;
}

function compareBytes(a: Uint8Array, b: Uint8Array) {
    const size = Math.min(a.length, b.length);
    for (let i = 0; i < size; i += 1) {
        if (a[i] !== b[i]) return a[i] < b[i] ? -1 : 1;
    }
    if (a.length === b.length) return 0;
    return a.length < b.length ? -1 : 1;
}

function u64Le(value: bigint) {
    const bytes = new Uint8Array(8);
    new DataView(bytes.buffer).setBigUint64(0, value, true);
    return bytes;
}

function hashPair(a: Uint8Array, b: Uint8Array) {
    const [left, right] = compareBytes(a, b) <= 0 ? [a, b] : [b, a];
    return keccak_256(concatBytes(left, right));
}

function scheduleBytes(schedule: VestingScheduleArgs) {
    return Uint8Array.from(vestingScheduleEncoder.encode(schedule));
}

function computeLeafHash(recipient: string, totalAmount: bigint, schedule: VestingScheduleArgs) {
    const claimantBytes = new PublicKey(recipient).toBytes();
    const innerHash = keccak_256(concatBytes(claimantBytes, u64Le(totalAmount), scheduleBytes(schedule)));
    return keccak_256(concatBytes(new Uint8Array([LEAF_PREFIX]), innerHash));
}

function buildMerkleTree(leaves: readonly Uint8Array[]) {
    const proofs = leaves.map((): Uint8Array[] => []);
    let level: MerkleNode[] = leaves
        .map((hash, index) => ({ hash, leafIndexes: [index] }))
        .sort((a, b) => compareBytes(a.hash, b.hash));

    while (level.length > 1) {
        const nextLevel: MerkleNode[] = [];
        for (let i = 0; i < level.length; i += 2) {
            const left = level[i];
            const right = level[i + 1];
            if (!right) {
                nextLevel.push(left);
                continue;
            }

            for (const leafIndex of left.leafIndexes) proofs[leafIndex].push(right.hash);
            for (const leafIndex of right.leafIndexes) proofs[leafIndex].push(left.hash);

            nextLevel.push({
                hash: hashPair(left.hash, right.hash),
                leafIndexes: [...left.leafIndexes, ...right.leafIndexes],
            });
        }
        level = nextLevel;
    }

    return { proofs, root: level[0]?.hash ?? new Uint8Array(32) };
}

function amountString(value: unknown, label: string): Result<string> {
    if (typeof value === 'string') return { ok: true, value: value.trim() };
    if (typeof value === 'number' && Number.isSafeInteger(value)) return { ok: true, value: String(value) };
    return { error: `${label} must be a string integer.`, ok: false };
}

function parseJsonRecipients(
    raw: string,
    sharedSchedule: VestingScheduleState,
): Result<ProofDropRecipientDraft[]> | null {
    let parsed: unknown;
    try {
        parsed = JSON.parse(raw);
    } catch {
        return null;
    }

    const rows = Array.isArray(parsed)
        ? parsed
        : isRecord(parsed) && Array.isArray(parsed.recipients)
          ? parsed.recipients
          : null;
    if (!rows) return null;

    const recipients: ProofDropRecipientDraft[] = [];
    for (const [index, row] of rows.entries()) {
        if (!isRecord(row)) return { error: `Recipient ${index + 1} must be an object.`, ok: false };

        const recipient = row.recipient ?? row.address ?? row.wallet;
        if (typeof recipient !== 'string') return { error: `Recipient ${index + 1} needs an address.`, ok: false };

        const totalAmount = amountString(row.totalAmount ?? row.amount, `Recipient ${index + 1} amount`);
        if (!totalAmount.ok) return totalAmount;

        const schedule = row.schedule === undefined ? sharedSchedule : row.schedule;
        if (!isVestingScheduleState(schedule))
            return { error: `Recipient ${index + 1} has an invalid schedule.`, ok: false };

        recipients.push({ recipient: recipient.trim(), schedule, totalAmount: totalAmount.value });
    }

    return { ok: true, value: recipients };
}

export function parseProofDropRecipients(
    raw: string,
    sharedSchedule: VestingScheduleState,
): Result<ProofDropRecipientDraft[]> {
    const normalized = raw.trim();
    if (!normalized) return { error: 'Add at least one recipient allocation.', ok: false };

    const jsonRecipients = parseJsonRecipients(normalized, sharedSchedule);
    if (jsonRecipients) return jsonRecipients;

    const recipients: ProofDropRecipientDraft[] = [];
    const lines = normalized
        .split(/\r?\n/)
        .map(line => line.trim())
        .filter(Boolean);

    for (const [index, line] of lines.entries()) {
        if (index === 0 && /^recipient\s*[, \t]\s*amount$/i.test(line)) continue;

        const parts = line.split(/[,\s]+/).filter(Boolean);
        if (parts.length !== 2) {
            return { error: `Allocation ${index + 1} must contain a recipient and amount.`, ok: false };
        }

        recipients.push({ recipient: parts[0], schedule: sharedSchedule, totalAmount: parts[1] });
    }

    if (recipients.length === 0) return { error: 'Add at least one recipient allocation.', ok: false };
    return { ok: true, value: recipients };
}

function validateRecipientDrafts(recipients: readonly ProofDropRecipientDraft[]) {
    const seenRecipients = new Set<string>();
    const parsedRecipients: Array<
        ProofDropRecipientDraft & { scheduleArgs: VestingScheduleArgs; totalAmountBigint: bigint }
    > = [];

    for (const [index, recipient] of recipients.entries()) {
        const rowLabel = `Recipient ${index + 1}`;
        const addressError = validateAddress(recipient.recipient, `${rowLabel} address`);
        if (addressError) return { error: addressError, ok: false } as const;

        const normalizedRecipient = normalize(recipient.recipient);
        if (seenRecipients.has(normalizedRecipient)) {
            return { error: `${rowLabel} duplicates ${normalizedRecipient}.`, ok: false } as const;
        }
        seenRecipients.add(normalizedRecipient);

        const amountError = validatePositiveInteger(recipient.totalAmount, `${rowLabel} amount`);
        if (amountError) return { error: amountError, ok: false } as const;

        const totalAmount = parseBigIntValue(recipient.totalAmount);
        if (totalAmount > MAX_U64) return { error: `${rowLabel} amount exceeds u64.`, ok: false } as const;

        const schedule = buildVestingSchedule(recipient.schedule);
        if (!schedule.ok) return { error: `${rowLabel}: ${schedule.error}`, ok: false } as const;

        parsedRecipients.push({
            recipient: normalizedRecipient,
            schedule: recipient.schedule,
            scheduleArgs: schedule.value,
            totalAmount: normalize(recipient.totalAmount),
            totalAmountBigint: totalAmount,
        });
    }

    return { ok: true, value: parsedRecipients } as const;
}

export function buildProofDropBundle({
    distribution,
    mint,
    recipients,
}: BuildProofDropBundleInput): Result<BuiltProofDropBundle> {
    const parsedRecipients = validateRecipientDrafts(recipients);
    if (!parsedRecipients.ok) return parsedRecipients;

    if (parsedRecipients.value.length === 0) return { error: 'Add at least one recipient allocation.', ok: false };

    const leaves = parsedRecipients.value.map(recipient =>
        computeLeafHash(recipient.recipient, recipient.totalAmountBigint, recipient.scheduleArgs),
    );
    const { proofs, root } = buildMerkleTree(leaves);
    const totalAmount = parsedRecipients.value.reduce((sum, recipient) => sum + recipient.totalAmountBigint, 0n);
    if (totalAmount > MAX_U64) return { error: 'Total allocation amount exceeds u64.', ok: false };

    const bundle: ProofDropBundle = {
        ...(distribution ? { distribution: normalize(distribution) } : {}),
        kind: 'proof-drop',
        merkleRoot: bytesToArray(root),
        merkleRootHex: bytesToHex(root),
        ...(mint ? { mint: normalize(mint) } : {}),
        recipients: parsedRecipients.value.map((recipient, index) => ({
            proof: proofs[index].map(bytesToArray),
            recipient: recipient.recipient,
            schedule: recipient.schedule,
            totalAmount: recipient.totalAmount,
        })),
        totalAmount: String(totalAmount),
        version: 1,
    };

    return {
        ok: true,
        value: {
            bundle,
            merkleRoot: bundle.merkleRoot,
            merkleRootHex: bundle.merkleRootHex,
            recipientCount: bundle.recipients.length,
            totalAmount,
        },
    };
}

function parseJson(raw: string): Result<unknown> {
    try {
        return { ok: true, value: JSON.parse(raw) as unknown };
    } catch {
        return { error: 'Proof bundle must be valid JSON.', ok: false };
    }
}

function resolveDistribution(candidate: unknown, options: ParseClaimBundleOptions): Result<string> {
    const bundleDistribution = typeof candidate === 'string' ? normalize(candidate) : '';
    const formDistribution = options.distribution ? normalize(options.distribution) : '';
    if (bundleDistribution && formDistribution && bundleDistribution !== formDistribution) {
        return { error: 'Proof bundle drop address does not match this drop.', ok: false };
    }

    const distribution = formDistribution || bundleDistribution;
    if (!distribution && options.requireDistribution) return { error: 'Proof bundle needs a drop address.', ok: false };
    if (!distribution) return { error: 'Drop address is required.', ok: false };

    const distributionError = validateAddress(distribution, 'Drop address');
    if (distributionError) return { error: distributionError, ok: false };
    return { ok: true, value: distribution };
}

function proofText(proof: readonly (readonly number[])[]) {
    return JSON.stringify(proof);
}

function proofsEqual(a: readonly (readonly number[])[], b: readonly (readonly number[])[]) {
    return proofText(a) === proofText(b);
}

function validateClaimRecipient(recipient: string, claimant?: string): Result<string> {
    const normalizedRecipient = normalize(recipient);
    const recipientError = validateAddress(normalizedRecipient, 'Recipient address');
    if (recipientError) return { error: recipientError, ok: false };

    if (claimant && normalizedRecipient !== normalize(claimant)) {
        return { error: 'Proof bundle does not contain the connected wallet.', ok: false };
    }

    return { ok: true, value: normalizedRecipient };
}

function claimFromRecipient(
    recipient: ProofDropBundleRecipient,
    distribution: string,
    claimant?: string,
): Result<ProofDropClaimDetails> {
    const recipientResult = validateClaimRecipient(recipient.recipient, claimant);
    if (!recipientResult.ok) return recipientResult;

    const amountError = validatePositiveInteger(recipient.totalAmount, 'Total allocation amount');
    if (amountError) return { error: amountError, ok: false };

    if (!isVestingScheduleState(recipient.schedule))
        return { error: 'Proof bundle has an invalid vesting schedule.', ok: false };
    if (!isMerkleProof(recipient.proof)) return { error: 'Proof bundle has an invalid proof.', ok: false };

    return {
        ok: true,
        value: {
            distribution,
            proof: recipient.proof,
            proofText: proofText(recipient.proof),
            recipient: recipientResult.value,
            schedule: recipient.schedule,
            totalAmount: normalize(recipient.totalAmount),
        },
    };
}

function parseFullBundle(
    value: Record<string, unknown>,
    options: ParseClaimBundleOptions,
): Result<ProofDropClaimDetails> {
    if (!Array.isArray(value.recipients)) return { error: 'Proof bundle needs recipients.', ok: false };

    const distribution = resolveDistribution(value.distribution, options);
    if (!distribution.ok) return distribution;

    const recipients: ProofDropBundleRecipient[] = [];
    for (const [index, row] of value.recipients.entries()) {
        if (!isRecord(row)) return { error: `Recipient ${index + 1} must be an object.`, ok: false };
        const recipient = row.recipient;
        const totalAmount = row.totalAmount;
        const schedule = row.schedule;
        const proof = row.proof;
        if (typeof recipient !== 'string') return { error: `Recipient ${index + 1} needs an address.`, ok: false };
        if (typeof totalAmount !== 'string') {
            return { error: `Recipient ${index + 1} total amount must be a string.`, ok: false };
        }
        if (!isVestingScheduleState(schedule))
            return { error: `Recipient ${index + 1} has an invalid schedule.`, ok: false };
        if (!isMerkleProof(proof)) return { error: `Recipient ${index + 1} has an invalid proof.`, ok: false };
        recipients.push({ proof, recipient, schedule, totalAmount });
    }

    const rebuilt = buildProofDropBundle({
        distribution: typeof value.distribution === 'string' ? value.distribution : undefined,
        mint: typeof value.mint === 'string' ? value.mint : undefined,
        recipients,
    });
    if (!rebuilt.ok) return rebuilt;

    if (isByteArray32(value.merkleRoot) && bytesToHex(value.merkleRoot) !== rebuilt.value.merkleRootHex) {
        return { error: 'Proof bundle root does not match its recipients.', ok: false };
    }
    if (
        typeof value.merkleRootHex === 'string' &&
        value.merkleRootHex.replace(/^0x/i, '').toLowerCase() !== rebuilt.value.merkleRootHex
    ) {
        return { error: 'Proof bundle root does not match its recipients.', ok: false };
    }

    const rebuiltByRecipient = new Map(
        rebuilt.value.bundle.recipients.map(recipient => [normalize(recipient.recipient), recipient] as const),
    );
    for (const recipient of recipients) {
        const rebuiltRecipient = rebuiltByRecipient.get(normalize(recipient.recipient));
        if (!rebuiltRecipient || !proofsEqual(recipient.proof, rebuiltRecipient.proof)) {
            return { error: 'Proof bundle proof does not match its recipients.', ok: false };
        }
    }

    const claimant = options.claimant ? normalize(options.claimant) : '';
    const matchingRecipient = claimant
        ? recipients.find(recipient => normalize(recipient.recipient) === claimant)
        : recipients.length === 1
          ? recipients[0]
          : null;

    if (!matchingRecipient) {
        return claimant
            ? { error: 'Proof bundle does not contain the connected wallet.', ok: false }
            : { error: 'Recipient address is required for this proof bundle.', ok: false };
    }

    return claimFromRecipient(matchingRecipient, distribution.value, claimant || undefined);
}

function parseSingleClaim(
    value: Record<string, unknown>,
    options: ParseClaimBundleOptions,
): Result<ProofDropClaimDetails> {
    const distribution = resolveDistribution(value.distribution, options);
    if (!distribution.ok) return distribution;

    const recipientValue = value.recipient ?? value.claimant ?? options.claimant;
    if (typeof recipientValue !== 'string') return { error: 'Proof bundle needs a recipient address.', ok: false };

    const totalAmount = value.totalAmount;
    const schedule = value.schedule;
    const proof = value.proof;
    if (typeof totalAmount !== 'string') return { error: 'Proof bundle total amount must be a string.', ok: false };
    if (!isVestingScheduleState(schedule)) return { error: 'Proof bundle has an invalid vesting schedule.', ok: false };
    if (!isMerkleProof(proof)) return { error: 'Proof bundle has an invalid proof.', ok: false };

    return claimFromRecipient(
        { proof, recipient: recipientValue, schedule, totalAmount },
        distribution.value,
        options.claimant,
    );
}

export function parseProofDropClaimBundle(
    raw: string,
    options: ParseClaimBundleOptions = {},
): Result<ProofDropClaimDetails> {
    const normalized = raw.trim();
    if (!normalized) return { error: 'Proof bundle is required.', ok: false };

    const parsed = parseJson(normalized);
    if (!parsed.ok) return parsed;
    if (!isRecord(parsed.value)) return { error: 'Proof bundle must be a JSON object.', ok: false };

    if (parsed.value.kind === 'proof-drop' || Array.isArray(parsed.value.recipients)) {
        return parseFullBundle(parsed.value, options);
    }

    return parseSingleClaim(parsed.value, options);
}

export function proofDropBundleText(bundle: ProofDropBundle) {
    return JSON.stringify(bundle, null, 2);
}
