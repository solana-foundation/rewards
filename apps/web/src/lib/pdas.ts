import {
    type Address,
    address,
    getAddressEncoder,
    getProgramDerivedAddress,
    type ReadonlyUint8Array,
} from '@solana/kit';
import { ASSOCIATED_TOKEN_PROGRAM_ID, getProgramAddress, TOKEN_PROGRAM_ID } from './program';

const seedEncoder = new TextEncoder();
const addressEncoder = getAddressEncoder();

function textSeed(value: string) {
    return seedEncoder.encode(value);
}

function publicKeyBytes(value: string) {
    return addressEncoder.encode(address(value));
}

function deriveAddress(seeds: ReadonlyUint8Array[], programId: string) {
    return getProgramDerivedAddress({ programAddress: address(programId), seeds });
}

export async function deriveEventAuthority(programId = getProgramAddress()) {
    return (await deriveAddress([textSeed('event_authority')], programId))[0];
}

export function deriveDirectDistributionPda(
    mint: string,
    authority: string,
    seed: string,
    programId = getProgramAddress(),
) {
    return deriveAddress(
        [textSeed('direct_distribution'), publicKeyBytes(mint), publicKeyBytes(authority), publicKeyBytes(seed)],
        programId,
    );
}

export function deriveDirectRecipientPda(distribution: string, recipient: string, programId = getProgramAddress()) {
    return deriveAddress(
        [textSeed('direct_recipient'), publicKeyBytes(distribution), publicKeyBytes(recipient)],
        programId,
    );
}

export function deriveMerkleDistributionPda(
    mint: string,
    authority: string,
    seed: string,
    programId = getProgramAddress(),
) {
    return deriveAddress(
        [textSeed('merkle_distribution'), publicKeyBytes(mint), publicKeyBytes(authority), publicKeyBytes(seed)],
        programId,
    );
}

export function deriveMerkleClaimPda(parent: string, user: string, programId = getProgramAddress()) {
    return deriveAddress([textSeed('merkle_claim'), publicKeyBytes(parent), publicKeyBytes(user)], programId);
}

export function deriveRevocationPda(parent: string, user: string, programId = getProgramAddress()) {
    return deriveAddress([textSeed('revocation'), publicKeyBytes(parent), publicKeyBytes(user)], programId);
}

export async function deriveAta(owner: string, mint: string, tokenProgram = TOKEN_PROGRAM_ID) {
    return (
        await deriveAddress(
            [publicKeyBytes(owner), publicKeyBytes(tokenProgram), publicKeyBytes(mint)],
            ASSOCIATED_TOKEN_PROGRAM_ID,
        )
    )[0];
}

export function normalizeTokenProgram(value: string) {
    return (value.trim() || TOKEN_PROGRAM_ID) as Address;
}
