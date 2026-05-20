import type { Address } from '@solana/kit';
import { PublicKey } from '@solana/web3.js';
import { ASSOCIATED_TOKEN_PROGRAM_ID, getProgramAddress, TOKEN_PROGRAM_ID } from './program';

const seedEncoder = new TextEncoder();

function pk(value: string) {
    return new PublicKey(value);
}

function textSeed(value: string) {
    return seedEncoder.encode(value);
}

function publicKeyBytes(value: string) {
    return pk(value).toBytes();
}

function deriveAddress(seeds: Uint8Array[], programId: string) {
    const [address, bump] = PublicKey.findProgramAddressSync(seeds, pk(programId));
    return [address.toBase58(), bump] as const;
}

export function deriveEventAuthority(programId = getProgramAddress()) {
    return deriveAddress([textSeed('event_authority')], programId)[0] as Address;
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

export function deriveAta(owner: string, mint: string, tokenProgram = TOKEN_PROGRAM_ID) {
    return deriveAddress(
        [publicKeyBytes(owner), publicKeyBytes(tokenProgram), publicKeyBytes(mint)],
        ASSOCIATED_TOKEN_PROGRAM_ID,
    )[0] as Address;
}

export function normalizeTokenProgram(value: string) {
    return (value.trim() || TOKEN_PROGRAM_ID) as Address;
}
