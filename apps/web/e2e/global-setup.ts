/**
 * Playwright global setup — runs once before all tests.
 *
 * Creates an SPL token mint on devnet and mints tokens to the test wallet so
 * the transaction tests have sufficient funds. The mint address is written to
 * e2e/.e2e-state.json and reused on subsequent runs (idempotent).
 *
 * Requires PLAYRIGHT_WALLET (base58 secret key) in .env at the repo root.
 */
import * as dotenv from 'dotenv';
import * as fs from 'fs';
import * as path from 'path';
import { Connection, Keypair, LAMPORTS_PER_SOL, PublicKey } from '@solana/web3.js';
import { createMint, getOrCreateAssociatedTokenAccount, mintTo } from '@solana/spl-token';
import bs58 from 'bs58';

dotenv.config({ path: path.resolve(__dirname, '../../../.env') });

const STATE_FILE = path.join(__dirname, '.e2e-state.json');
const DEVNET_URL = 'https://api.devnet.solana.com';
const DECIMALS = 9;
const MINT_AMOUNT = BigInt(100_000_000_000_000); // 100,000 tokens

interface E2EState {
    walletPubkey: string;
    mint: string;
}

async function sleep(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function airdropWithRetry(connection: Connection, keypair: Keypair, lamports: number, retries = 5) {
    for (let i = 0; i < retries; i++) {
        try {
            const sig = await connection.requestAirdrop(keypair.publicKey, lamports);
            const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();
            await connection.confirmTransaction({ blockhash, lastValidBlockHeight, signature: sig }, 'confirmed');
            return;
        } catch (err) {
            if (i === retries - 1) throw err;
            console.log(`  airdrop attempt ${i + 1} failed, retrying...`);
            await sleep(8_000);
        }
    }
}

export default async function globalSetup() {
    const secretKeyB58 = process.env.PLAYRIGHT_WALLET;
    if (!secretKeyB58) throw new Error('PLAYRIGHT_WALLET is not set in .env');

    const secretKey = bs58.decode(secretKeyB58);
    const keypair = Keypair.fromSecretKey(secretKey);
    const walletPubkey = keypair.publicKey.toBase58();

    console.log(`\n[e2e setup] wallet: ${walletPubkey}`);

    // Reuse existing state if wallet matches, but always ensure dummy ATA exists.
    if (fs.existsSync(STATE_FILE)) {
        const existing = JSON.parse(fs.readFileSync(STATE_FILE, 'utf-8')) as Partial<E2EState>;
        if (existing.walletPubkey === walletPubkey && existing.mint) {
            console.log(`[e2e setup] reusing mint ${existing.mint}`);
            const connection = new Connection(DEVNET_URL, 'confirmed');
            const dummyRecipient = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');
            const mint = new PublicKey(existing.mint);
            const dummyAta = await getOrCreateAssociatedTokenAccount(connection, keypair, mint, dummyRecipient);
            console.log(`[e2e setup] dummy recipient ATA: ${dummyAta.address.toBase58()}\n`);
            return;
        }
    }

    const connection = new Connection(DEVNET_URL, 'confirmed');

    const balance = await connection.getBalance(keypair.publicKey);
    console.log(`[e2e setup] SOL balance: ${(balance / LAMPORTS_PER_SOL).toFixed(3)}`);
    if (balance < 1 * LAMPORTS_PER_SOL) {
        console.log('[e2e setup] requesting 2 SOL airdrop...');
        await airdropWithRetry(connection, keypair, 2 * LAMPORTS_PER_SOL);
        console.log('[e2e setup] airdrop confirmed');
    }

    console.log('[e2e setup] creating mint...');
    const mint = await createMint(connection, keypair, keypair.publicKey, keypair.publicKey, DECIMALS);
    console.log(`[e2e setup] mint: ${mint.toBase58()}`);

    const ata = await getOrCreateAssociatedTokenAccount(connection, keypair, mint, keypair.publicKey);
    await mintTo(connection, keypair, mint, ata.address, keypair.publicKey, MINT_AMOUNT);
    console.log(`[e2e setup] minted ${MINT_AMOUNT} base units to ${ata.address.toBase58()}`);

    // Create ATA for the DUMMY_RECIPIENT (TOKEN_PROGRAM address) so that
    // RevokeDirectRecipient's verify_owned_by(recipientTokenAccount) check passes.
    // This is needed because the deployed program validates the recipient's token account
    // even when 0 tokens are being returned (NonVested with Immediate vesting).
    const dummyRecipient = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');
    const dummyAta = await getOrCreateAssociatedTokenAccount(connection, keypair, mint, dummyRecipient);
    console.log(`[e2e setup] dummy recipient ATA: ${dummyAta.address.toBase58()}\n`);

    const state: E2EState = { walletPubkey, mint: mint.toBase58() };
    fs.writeFileSync(STATE_FILE, JSON.stringify(state, null, 2));
}
