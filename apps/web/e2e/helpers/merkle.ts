/**
 * TypeScript port of the on-chain merkle utilities for computing leaf hashes
 * and building single-leaf proofs used in E2E tests.
 *
 * Hash function: Keccak-256 (NOT SHA3-256 — different padding).
 * Source: tests/integration-tests/src/utils/merkle_utils.rs
 */
import { keccak_256 } from '@noble/hashes/sha3';

const LEAF_PREFIX = new Uint8Array([0]);

function keccak256(data: Uint8Array): Uint8Array {
    return keccak_256(data);
}

function bigintToLeBytes(value: bigint): Uint8Array {
    const buf = new Uint8Array(8);
    let v = value;
    for (let i = 0; i < 8; i++) {
        buf[i] = Number(v & 0xffn);
        v >>= 8n;
    }
    return buf;
}

function bytesToHex(bytes: Uint8Array): string {
    return Array.from(bytes)
        .map(b => b.toString(16).padStart(2, '0'))
        .join('');
}

function b58Decode(s: string): Uint8Array {
    const ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
    const bytes = [0];
    for (const c of s) {
        const idx = ALPHABET.indexOf(c);
        if (idx < 0) throw new Error('Invalid base58 char: ' + c);
        let carry = idx;
        for (let j = 0; j < bytes.length; j++) {
            carry += bytes[j] * 58;
            bytes[j] = carry & 0xff;
            carry >>= 8;
        }
        while (carry > 0) {
            bytes.push(carry & 0xff);
            carry >>= 8;
        }
    }
    for (const c of s) {
        if (c === '1') bytes.push(0);
        else break;
    }
    return new Uint8Array(bytes.reverse());
}

function outerHash(innerHash: Uint8Array): Uint8Array {
    const data = new Uint8Array(33);
    data.set(LEAF_PREFIX, 0);
    data.set(innerHash, 1);
    return keccak256(data);
}

/**
 * Leaf hash for a direct/merkle distribution claim (Immediate vesting).
 * Returns the 32-byte root hex string — for a single-leaf tree, root = leaf.
 */
export function merkleRootForClaim(claimantB58: string, totalAmount: bigint): string {
    const claimantBytes = b58Decode(claimantB58);
    // inner = claimant(32) + total_amount_le64(8) + schedule_immediate(1)
    const inner = new Uint8Array(41);
    inner.set(claimantBytes, 0);
    inner.set(bigintToLeBytes(totalAmount), 32);
    inner[40] = 0; // VestingSchedule::Immediate
    return bytesToHex(outerHash(keccak256(inner)));
}

/**
 * Leaf hash for a continuous cumulative claim.
 * Returns the 32-byte root hex for a single-leaf tree.
 */
export function merkleRootForContinuousClaim(
    rewardPoolB58: string,
    claimantB58: string,
    rootVersion: bigint,
    cumulativeAmount: bigint,
): string {
    const poolBytes = b58Decode(rewardPoolB58);
    const claimantBytes = b58Decode(claimantB58);
    // inner = pool(32) + claimant(32) + root_version_le64(8) + cumulative_amount_le64(8)
    const inner = new Uint8Array(80);
    inner.set(poolBytes, 0);
    inner.set(claimantBytes, 32);
    inner.set(bigintToLeBytes(rootVersion), 64);
    inner.set(bigintToLeBytes(cumulativeAmount), 72);
    return bytesToHex(outerHash(keccak256(inner)));
}
