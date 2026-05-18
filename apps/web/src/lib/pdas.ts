import type { Address } from '@solana/kit';
import { PublicKey } from '@solana/web3.js';
import { ASSOCIATED_TOKEN_PROGRAM_ID, getProgramAddress, TOKEN_PROGRAM_ID } from './program';

function pk(value: string) {
    return new PublicKey(value);
}

function deriveAddress(seeds: Buffer[], programId: string) {
    const [address, bump] = PublicKey.findProgramAddressSync(seeds, pk(programId));
    return [address.toBase58(), bump] as const;
}

export function deriveEventAuthority(programId = getProgramAddress()) {
    return deriveAddress([Buffer.from('event_authority')], programId)[0] as Address;
}

export function deriveDirectDistributionPda(
    mint: string,
    authority: string,
    seed: string,
    programId = getProgramAddress(),
) {
    return deriveAddress(
        [Buffer.from('direct_distribution'), pk(mint).toBuffer(), pk(authority).toBuffer(), pk(seed).toBuffer()],
        programId,
    );
}

export function deriveDirectRecipientPda(distribution: string, recipient: string, programId = getProgramAddress()) {
    return deriveAddress(
        [Buffer.from('direct_recipient'), pk(distribution).toBuffer(), pk(recipient).toBuffer()],
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
        [Buffer.from('merkle_distribution'), pk(mint).toBuffer(), pk(authority).toBuffer(), pk(seed).toBuffer()],
        programId,
    );
}

export function deriveMerkleClaimPda(parent: string, user: string, programId = getProgramAddress()) {
    return deriveAddress([Buffer.from('merkle_claim'), pk(parent).toBuffer(), pk(user).toBuffer()], programId);
}

export function deriveRevocationPda(parent: string, user: string, programId = getProgramAddress()) {
    return deriveAddress([Buffer.from('revocation'), pk(parent).toBuffer(), pk(user).toBuffer()], programId);
}

export function deriveAta(owner: string, mint: string, tokenProgram = TOKEN_PROGRAM_ID) {
    return deriveAddress(
        [pk(owner).toBuffer(), pk(tokenProgram).toBuffer(), pk(mint).toBuffer()],
        ASSOCIATED_TOKEN_PROGRAM_ID,
    )[0] as Address;
}

export function normalizeTokenProgram(value: string) {
    return (value.trim() || TOKEN_PROGRAM_ID) as Address;
}
