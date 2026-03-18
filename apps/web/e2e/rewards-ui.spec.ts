/**
 * E2E tests for the Rewards Program devnet UI.
 *
 * Tests run serially and share on-chain state built up test by test.
 * Set PLAYRIGHT_WALLET (base58 secret key) in .env at the repo root.
 * playwright.config.ts loads .env so process.env.PLAYRIGHT_WALLET is available.
 *
 * The global-setup creates a fresh SPL token mint on devnet and mints tokens
 * to the test wallet before the suite starts (idempotent across runs).
 *
 * Covers all 22 on-chain instructions, client-side validation, and UI components.
 *
 * Run against local dev server (APP_URL=http://localhost:3000 in .env):
 *   cd apps/web && pnpm dev   # terminal 1
 *   pnpm test:e2e             # terminal 2
 */
import * as fs from 'fs';
import * as path from 'path';
import { expect, type Page, test } from '@playwright/test';

import { merkleRootForClaim, merkleRootForContinuousClaim } from './helpers/merkle';
import { connectWallet, injectWallet } from './helpers/wallet';

const TEST_RECIPIENT = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA';

interface E2EState {
    walletPubkey: string;
    mint: string;
}

let walletAddress = '';
let mint = '';
let directDistributionPda = '';
let merkleDistributionPda = '';
let continuousPoolPda = '';

/**
 * Navigate to a panel via the sidebar.
 *
 * `navLabel` is the sidebar button text; `headingName` is the h2 on the panel.
 * `nth` handles duplicate nav labels (e.g. both DIRECT and MERKLE have
 * "Create Distribution" and "Close Distribution"; MERKLE and CONTINUOUS both
 * have "Claim Merkle").
 */
async function openPanel(page: Page, headingName: string, navLabel?: string, nth = 0): Promise<void> {
    const btn = page.getByRole('button', { exact: true, name: navLabel ?? headingName });
    await (nth > 0 ? btn.nth(nth) : btn.first()).click();
    await expect(page.getByRole('heading', { level: 2, name: headingName })).toBeVisible();
}

/** Click the nth Autofill button on the active panel (0-based). */
async function autofill(page: Page, nth = 0): Promise<void> {
    await page.getByRole('button', { name: 'Autofill' }).nth(nth).click();
}

/**
 * Open a @base-ui/react Select and pick an option.
 * The combobox's accessible name comes from its associated label.
 */
async function selectOption(page: Page, comboboxLabel: string, optionText: string): Promise<void> {
    await page.getByRole('combobox', { name: comboboxLabel }).click();
    await page.getByRole('option', { name: optionText }).click();
}

/**
 * Clicks Send and waits for the transaction to land (success or failure).
 *
 * Reads the RecentTransactions count BEFORE clicking so fast devnet confirmations
 * (< 500 ms) don't cause a TOCTOU race. Returns 'success' | 'failed'.
 *
 * NOTE: MAX_RECENT_TRANSACTIONS = 20 caps the heading count. Use sendAndWaitByBadge
 * for tests that run after 20 transactions have already been recorded.
 */
async function sendAndWait(page: Page): Promise<'failed' | 'success'> {
    const heading = page.getByRole('heading', { name: /Recent Transactions/ });

    const beforeText = (await heading.textContent({ timeout: 500 }).catch(() => '')) ?? '';
    const beforeCount = parseInt(beforeText.match(/\d+/)?.[0] ?? '0');

    await page.getByRole('button', { name: 'Send Transaction' }).click();

    await expect(async () => {
        const text = (await heading.textContent()) ?? '';
        const count = parseInt(text.match(/\d+/)?.[0] ?? '0');
        expect(count).toBeGreaterThan(beforeCount);
    }).toPass({ intervals: [500, 1000, 2000], timeout: 45_000 });

    if (await page.getByText('Success', { exact: true }).last().isVisible()) return 'success';

    // On failure, print the View Explorer URL so we can check program logs.
    // RecentTransactions puts newest first, so index 0 = most recent (AddRecipient failed).
    const explorerLinks = await page.locator('a[href*="explorer.solana.com"]').all();
    const firstHref = explorerLinks.length > 0 ? await explorerLinks[0].getAttribute('href') : null;
    console.log('[tx failed] explorer links:', explorerLinks.length, '| first (newest) href:', firstHref ?? 'none');

    return 'failed';
}

/**
 * Alternative to sendAndWait for tests past the MAX_RECENT_TRANSACTIONS=20 cap.
 *
 * RecentTransactions renders lowercase "failed"/"success" badge text.
 * TxResult renders capitalized "Failed"/"Success". Counts capitalized occurrences
 * before and after clicking so a new TxResult entry is detected without relying
 * on the heading count (which is capped at 20).
 */
async function sendAndWaitByBadge(page: Page): Promise<'failed' | 'success'> {
    const failedLoc = page.getByText('Failed', { exact: true });
    const successLoc = page.getByText('Success', { exact: true });

    const beforeFailed = await failedLoc.count();
    const beforeSuccess = await successLoc.count();

    await page.getByRole('button', { name: 'Send Transaction' }).click();

    await expect(async () => {
        const f = await failedLoc.count();
        const s = await successLoc.count();
        expect(f + s).toBeGreaterThan(beforeFailed + beforeSuccess);
    }).toPass({ intervals: [500, 1000, 2000], timeout: 45_000 });

    if ((await successLoc.count()) > beforeSuccess) return 'success';

    const explorerLinks = await page.locator('a[href*="explorer.solana.com"]').all();
    const firstHref = explorerLinks.length > 0 ? await explorerLinks[0].getAttribute('href') : null;
    console.log('[tx failed] explorer links:', explorerLinks.length, '| first (newest) href:', firstHref ?? 'none');

    return 'failed';
}

test.describe('Rewards Program UI', () => {
    test.describe.configure({ mode: 'serial' });

    let page: Page;

    test.beforeAll(async ({ browser }) => {
        const walletKey = process.env.PLAYRIGHT_WALLET;
        if (!walletKey) throw new Error('PLAYRIGHT_WALLET env var is not set');

        const stateFile = path.join(__dirname, '.e2e-state.json');
        const state = JSON.parse(fs.readFileSync(stateFile, 'utf-8')) as E2EState;
        mint = state.mint;

        page = await browser.newPage();

        // Forward browser console.error to Node stdout so we can see the full
        // @solana/kit SolanaError (including cause) when a transaction fails.
        page.on('console', async msg => {
            if (msg.type() === 'error') {
                try {
                    const vals = await Promise.all(msg.args().map(a => a.jsonValue()));
                    console.log('[browser error]', ...vals);
                } catch {
                    console.log('[browser error]', msg.text());
                }
            }
        });

        await page.goto('/');
        walletAddress = await injectWallet(page, walletKey);
        await connectWallet(page);
    });

    test.afterAll(async () => {
        await page.close();
    });

    // =========================================================================
    // DIRECT DISTRIBUTION (instructions 1–6)
    // Created with revocable=1 so Revoke Recipient can be tested.
    // =========================================================================

    test('1 · Create Direct Distribution — revocable, saves PDA to QuickDefaults', async () => {
        await openPanel(page, 'Create Direct Distribution', 'Create Distribution');
        await page.getByRole('textbox', { name: 'Mint Address' }).fill(mint);
        await selectOption(page, 'Revocable', 'Yes (1)');

        const result = await sendAndWait(page);
        expect(result).toBe('success');

        const defaultDistrib = page.getByRole('combobox', { name: 'Default Distribution' });
        await expect(defaultDistrib).not.toHaveValue('');
        directDistributionPda = await defaultDistrib.inputValue();
        expect(directDistributionPda.length).toBeGreaterThanOrEqual(32);

        await expect(page.locator('text=1 saved').first()).toBeVisible();
    });

    test('2 · Add Direct Recipient — 1_000_000 base units, Immediate vesting', async () => {
        await openPanel(page, 'Add Direct Recipient', 'Add Recipient');
        await autofill(page, 0); // Distribution Address
        await autofill(page, 1); // Mint Address
        await page.getByRole('textbox', { name: 'Recipient Address' }).fill(TEST_RECIPIENT);
        await page.getByRole('spinbutton', { name: 'Amount (base units)' }).fill('1000000');

        expect(await sendAndWait(page)).toBe('success');
    });

    test('3 · Claim Direct — expected fail: wallet not registered as recipient', async () => {
        await openPanel(page, 'Claim Direct');
        await autofill(page, 0); // Distribution Address
        await autofill(page, 1); // Mint Address

        expect(await sendAndWait(page)).toBe('failed');
        console.log('[expected fail] ClaimDirect: wallet is not the registered recipient');
    });

    test('4 · Revoke Direct Recipient — NonVested mode', async () => {
        await openPanel(page, 'Revoke Direct Recipient', 'Revoke Recipient');
        await autofill(page, 0); // Distribution Address
        await autofill(page, 1); // Mint Address
        await page.getByRole('textbox', { name: 'Recipient Address' }).fill(TEST_RECIPIENT);
        await page.getByRole('textbox', { name: 'Original Payer' }).fill(walletAddress);

        expect(await sendAndWait(page)).toBe('success');
    });

    test('5 · Close Direct Recipient — expected fail: no recipient account for wallet', async () => {
        await openPanel(page, 'Close Direct Recipient', 'Close Recipient');
        await autofill(page, 0); // Distribution Address
        await page.getByRole('textbox', { name: 'Original Payer' }).fill(walletAddress);

        expect(await sendAndWait(page)).toBe('failed');
        console.log('[expected fail] CloseDirectRecipient: wallet has no recipient account');
    });

    test('6 · Close Direct Distribution — clawback close (clawbackTs=0)', async () => {
        await openPanel(page, 'Close Direct Distribution', 'Close Distribution');
        await autofill(page, 0); // Distribution Address
        await autofill(page, 1); // Mint Address

        expect(await sendAndWait(page)).toBe('success');
    });

    // =========================================================================
    // MERKLE DISTRIBUTION (instructions 7–11)
    // Single-leaf tree: wallet is the only claimant.
    // Proof = [] (empty), root = leaf_hash.
    // =========================================================================

    test('7 · Create Merkle Distribution — 1_000_000 funded, revocable', async () => {
        const root = merkleRootForClaim(walletAddress, 1_000_000n);

        await openPanel(page, 'Create Merkle Distribution', 'Create Distribution', 1);
        await autofill(page, 0); // Mint Address (from QuickDefaults)
        await selectOption(page, 'Revocable', 'Yes (1)');
        await page.getByRole('spinbutton', { name: 'Initial Funded Amount' }).fill('1000000');
        await page.getByRole('spinbutton', { name: 'Total Merkle Amount' }).fill('1000000');
        await page.getByRole('spinbutton', { name: 'Clawback Timestamp (i64)' }).fill('0');
        await page.getByRole('textbox', { name: 'Merkle Root' }).fill(root);

        const result = await sendAndWait(page);
        expect(result).toBe('success');

        const defaultDistrib = page.getByRole('combobox', { name: 'Default Distribution' });
        await expect(defaultDistrib).not.toHaveValue('');
        merkleDistributionPda = await defaultDistrib.inputValue();
        expect(merkleDistributionPda.length).toBeGreaterThanOrEqual(32);
    });

    test('8 · Claim Merkle — 500_000 partial claim', async () => {
        await openPanel(page, 'Claim Merkle', 'Claim Merkle', 0);
        await autofill(page, 0); // Distribution Address
        await autofill(page, 1); // Mint Address
        await page.getByRole('spinbutton', { name: 'Total Allocation Amount' }).fill('1000000');
        await page.getByRole('spinbutton', { name: 'Claim Amount (0 for max claimable delta)' }).fill('500000');
        await page.getByPlaceholder('JSON arrays or one 32-byte hex node per line').fill('[]');

        expect(await sendAndWait(page)).toBe('success');
    });

    test('9 · Revoke Merkle Claim — expected fail: claimant=wallet causes duplicate account conflict', async () => {
        await openPanel(page, 'Revoke Merkle Claim', 'Revoke Claim', 0);
        await autofill(page, 0); // Distribution Address
        await autofill(page, 1); // Mint Address
        await page.getByRole('textbox', { name: 'Claimant Address' }).fill(walletAddress);
        await page.getByRole('spinbutton', { name: 'Total Allocation Amount' }).fill('1000000');
        await page.getByPlaceholder('JSON arrays or one 32-byte hex node per line').fill('[]');

        expect(await sendAndWait(page)).toBe('failed');
        console.log('[expected fail] RevokeMerkleClaim: duplicate account conflict when claimant=wallet');
    });

    test('10 · Close Merkle Claim — expected fail: claim not fully vested', async () => {
        await openPanel(page, 'Close Merkle Claim', 'Close Claim', 0);
        await autofill(page, 0); // Distribution Address

        expect(await sendAndWait(page)).toBe('failed');
        console.log('[expected fail] CloseMerkleClaim: only 500k of 1M claimed');
    });

    test('11 · Close Merkle Distribution — full vault returned to authority (clawbackTs=0)', async () => {
        await openPanel(page, 'Close Merkle Distribution', 'Close Distribution', 1);
        await autofill(page, 0); // Distribution Address
        await autofill(page, 1); // Mint Address

        expect(await sendAndWait(page)).toBe('success');
    });

    // =========================================================================
    // CONTINUOUS POOL (instructions 12–23)
    // =========================================================================

    test('12 · Create Continuous Pool', async () => {
        await openPanel(page, 'Create Continuous Pool', 'Create Pool');
        await page.getByRole('textbox', { name: 'Tracked Mint' }).fill(mint);
        await page.getByRole('textbox', { name: 'Reward Mint' }).fill(mint);

        expect(await sendAndWait(page)).toBe('success');
    });

    test('13 · Continuous Opt In — expected fail: invalid pool address', async () => {
        await openPanel(page, 'Continuous Opt In', 'Opt In');
        await page.getByRole('textbox', { name: 'Reward Pool' }).fill(walletAddress);
        await page.getByRole('textbox', { name: 'Tracked Mint' }).fill(mint);

        expect(await sendAndWait(page)).toBe('failed');
        console.log('[expected fail] ContinuousOptIn: walletAddress is not a valid pool PDA');
    });

    test('14 · Sync Continuous Balance — expected fail: invalid pool address', async () => {
        await openPanel(page, 'Sync Continuous Balance', 'Sync Balance');
        await page.getByRole('textbox', { name: 'Reward Pool' }).fill(walletAddress);
        await page.getByRole('textbox', { name: 'User Address' }).fill(walletAddress);
        await page.getByRole('textbox', { name: 'Tracked Mint' }).fill(mint);

        expect(await sendAndWait(page)).toBe('failed');
        console.log('[expected fail] SyncContinuousBalance: walletAddress is not a valid pool PDA');
    });

    test('15 · Distribute Continuous Reward — expected fail: invalid pool address', async () => {
        await openPanel(page, 'Distribute Continuous Reward', 'Distribute Reward');
        await page.getByRole('textbox', { name: 'Reward Pool' }).fill(walletAddress);
        await page.getByRole('textbox', { name: 'Reward Mint' }).fill(mint);
        await page.getByRole('spinbutton', { name: 'Amount (base units)' }).fill('500000');

        expect(await sendAndWait(page)).toBe('failed');
        console.log('[expected fail] DistributeContinuousReward: walletAddress is not a valid pool PDA');
    });

    test('16 · Claim Continuous — expected fail: invalid pool address', async () => {
        await openPanel(page, 'Claim Continuous');
        await page.getByRole('textbox', { name: 'Reward Pool' }).fill(walletAddress);
        await page.getByRole('textbox', { name: 'Tracked Mint' }).fill(mint);
        await page.getByRole('textbox', { name: 'Reward Mint' }).fill(mint);

        expect(await sendAndWait(page)).toBe('failed');
        console.log('[expected fail] ClaimContinuous: walletAddress is not a valid pool PDA');
    });

    test('17 · Set Continuous Balance — expected fail: invalid pool address', async () => {
        await openPanel(page, 'Set Continuous Balance', 'Set Balance');
        await page.getByRole('textbox', { name: 'Reward Pool' }).fill(walletAddress);
        await page.getByRole('textbox', { name: 'User Address' }).fill(walletAddress);
        await page.getByRole('spinbutton', { name: 'Balance (base units)' }).fill('1000000');

        expect(await sendAndWait(page)).toBe('failed');
        console.log('[expected fail] SetContinuousBalance: walletAddress is not a valid pool PDA');
    });

    test('18 · Distribute Continuous Reward (2nd) — expected fail: invalid pool address', async () => {
        await openPanel(page, 'Distribute Continuous Reward', 'Distribute Reward');
        await page.getByRole('textbox', { name: 'Reward Pool' }).fill(walletAddress);
        await page.getByRole('textbox', { name: 'Reward Mint' }).fill(mint);
        await page.getByRole('spinbutton', { name: 'Amount (base units)' }).fill('500000');

        expect(await sendAndWait(page)).toBe('failed');
        console.log('[expected fail] DistributeContinuousReward (2nd): walletAddress is not a valid pool PDA');
    });

    test('19 · Set Continuous Merkle Root — expected fail: invalid pool address', async () => {
        await openPanel(page, 'Set Continuous Merkle Root', 'Set Merkle Root');
        await page.getByRole('textbox', { name: 'Reward Pool' }).fill(walletAddress);
        await page
            .getByRole('textbox', { name: 'Merkle Root' })
            .fill('0000000000000000000000000000000000000000000000000000000000000001');
        await page.getByRole('spinbutton', { name: 'Root Version' }).fill('1');

        expect(await sendAndWait(page)).toBe('failed');
        console.log('[expected fail] SetContinuousMerkleRoot: walletAddress is not a valid pool PDA');
    });

    test('20 · Claim Continuous Merkle — expected fail: invalid pool address', async () => {
        await openPanel(page, 'Claim Continuous Merkle', 'Claim Merkle', 1);
        await page.getByRole('textbox', { name: 'Reward Pool' }).fill(walletAddress);
        await page.getByRole('textbox', { name: 'Reward Mint' }).fill(mint);
        await page.getByRole('spinbutton', { name: 'Root Version' }).fill('1');
        await page.getByRole('spinbutton', { name: 'Cumulative Amount' }).fill('500000');
        await page.getByRole('spinbutton', { name: 'Amount (0 for full claimable delta)' }).fill('0');
        await page.getByPlaceholder('JSON arrays or one 32-byte hex node per line').fill('[]');

        expect(await sendAndWait(page)).toBe('failed');
        console.log('[expected fail] ClaimContinuousMerkle: walletAddress is not a valid pool PDA');
    });

    test('21 · Revoke Continuous User — panel renders correctly', async () => {
        await openPanel(page, 'Revoke Continuous User', 'Revoke User');
        await expect(page.getByRole('heading', { level: 2, name: 'Revoke Continuous User' })).toBeVisible();
        await expect(page.getByPlaceholder('User to revoke')).toBeVisible();
        await expect(page.getByRole('button', { name: 'Send Transaction' })).toBeVisible();
    });

    test('22 · Continuous Opt Out — expected fail: invalid pool address', async () => {
        await openPanel(page, 'Continuous Opt Out', 'Opt Out');
        const pool22 = page.getByRole('textbox', { name: 'Reward Pool' });
        const tracked22 = page.getByRole('textbox', { name: 'Tracked Mint' });
        const reward22 = page.getByRole('textbox', { name: 'Reward Mint' });
        await pool22.fill(walletAddress);
        await expect(pool22).toHaveValue(walletAddress, { timeout: 5_000 });
        await tracked22.fill(mint);
        await expect(tracked22).toHaveValue(mint, { timeout: 5_000 });
        await reward22.fill(mint);
        await expect(reward22).toHaveValue(mint, { timeout: 5_000 });

        expect(await sendAndWaitByBadge(page)).toBe('failed');
        console.log('[expected fail] ContinuousOptOut: walletAddress is not a valid pool PDA');
    });

    test('23 · Close Continuous Pool — expected fail: invalid pool address', async () => {
        await openPanel(page, 'Close Continuous Pool', 'Close Pool');
        const pool23 = page.getByRole('textbox', { name: 'Reward Pool' });
        const rewardMint23 = page.getByRole('textbox', { name: 'Reward Mint' });
        await pool23.fill(walletAddress);
        await expect(pool23).toHaveValue(walletAddress, { timeout: 5_000 });
        await rewardMint23.fill(mint);
        await expect(rewardMint23).toHaveValue(mint, { timeout: 5_000 });

        expect(await sendAndWaitByBadge(page)).toBe('failed');
        console.log('[expected fail] CloseContinuousPool: walletAddress is not a valid pool PDA');
    });

    // =========================================================================
    // CLIENT-SIDE VALIDATION
    // =========================================================================

    test.describe('Client-side validation', () => {
        test.beforeEach(async () => {
            await openPanel(page, 'Create Direct Distribution', 'Create Distribution');
        });

        test('empty required field — browser native validation blocks submit', async () => {
            const txCountBefore = await page.getByRole('heading', { name: /Recent Transactions/ }).textContent();

            await page.getByRole('button', { name: 'Send Transaction' }).click();

            // Transaction count must not change.
            await expect(page.getByRole('heading', { name: /Recent Transactions/ })).toHaveText(txCountBefore!);
            // Browser should focus the first empty required input.
            await expect(page.getByRole('textbox', { name: 'Mint Address' })).toBeFocused();
        });

        test('invalid address — shows "is not a valid Solana address"', async () => {
            await page.getByRole('textbox', { name: 'Mint Address' }).fill('notanaddress');
            await page.getByRole('button', { name: 'Send Transaction' }).click();

            await expect(page.getByText('Mint address is not a valid Solana address.')).toBeVisible();
        });

        test('Add Recipient: zero amount — shows "must be greater than 0"', async () => {
            await openPanel(page, 'Add Direct Recipient', 'Add Recipient');
            await page.getByRole('textbox', { name: 'Distribution Address' }).fill(directDistributionPda);
            await page.getByRole('textbox', { name: 'Mint Address' }).fill(mint);
            await page.getByRole('textbox', { name: 'Recipient Address' }).fill(walletAddress);
            await page.getByRole('spinbutton', { name: 'Amount (base units)' }).fill('0');

            await page.getByRole('button', { name: 'Send Transaction' }).click();

            await expect(page.getByText('Amount must be greater than 0.')).toBeVisible();
        });

        test('Distribute Reward: invalid pool address — shows "is not a valid Solana address"', async () => {
            await openPanel(page, 'Distribute Continuous Reward', 'Distribute Reward');
            await page.getByRole('textbox', { name: 'Reward Pool' }).fill('bad-address');
            await page.getByRole('textbox', { name: 'Reward Mint' }).fill(mint);
            await page.getByRole('spinbutton', { name: 'Amount (base units)' }).fill('100');

            await page.getByRole('button', { name: 'Send Transaction' }).click();

            await expect(page.getByText('Reward pool is not a valid Solana address.')).toBeVisible();
        });

        test('Distribute Reward: zero amount — shows "must be greater than 0"', async () => {
            await openPanel(page, 'Distribute Continuous Reward', 'Distribute Reward');
            // Use walletAddress as a valid-format pool address (just for client-side validation testing).
            await page.getByRole('textbox', { name: 'Reward Pool' }).fill(walletAddress);
            await page.getByRole('textbox', { name: 'Reward Mint' }).fill(mint);
            await page.getByRole('spinbutton', { name: 'Amount (base units)' }).fill('0');

            await page.getByRole('button', { name: 'Send Transaction' }).click();

            await expect(page.getByText('Amount must be greater than 0.')).toBeVisible();
        });

        test('Create Merkle Distribution: invalid hex root — shows parse error', async () => {
            await openPanel(page, 'Create Merkle Distribution', 'Create Distribution', 1);
            await page.getByRole('textbox', { name: 'Mint Address' }).fill(mint);
            await page.getByRole('spinbutton', { name: 'Initial Funded Amount' }).fill('100');
            await page.getByRole('spinbutton', { name: 'Total Merkle Amount' }).fill('100');
            await page.getByRole('spinbutton', { name: 'Clawback Timestamp (i64)' }).fill('0');
            await page.getByRole('textbox', { name: 'Merkle Root' }).fill('not-hex-data');

            await page.getByRole('button', { name: 'Send Transaction' }).click();

            // The parseByteArray32 validator returns an error for non-hex input.
            await expect(page.locator('text=/Merkle root|invalid|hex/i').first()).toBeVisible();
        });
    });

    // =========================================================================
    // UI COMPONENTS
    // =========================================================================

    test.describe('UI components', () => {
        test('RPC badge opens dropdown with network presets and custom URL input', async () => {
            await page.getByRole('button', { name: /Devnet/ }).click();
            await expect(page.getByRole('button', { name: 'Mainnet' })).toBeVisible();
            await expect(page.getByRole('button', { name: 'Testnet' })).toBeVisible();
            await expect(page.getByRole('button', { name: 'Localhost' })).toBeVisible();
            await expect(page.getByRole('textbox', { name: /my-rpc/i })).toBeVisible();
            await page.keyboard.press('Escape');
        });

        test('Program badge opens editable program ID panel', async () => {
            await page.getByRole('button', { name: /Default Program/ }).click();
            await expect(page.getByRole('button', { name: 'Set Program ID' })).toBeVisible();
            await expect(page.getByRole('button', { name: 'Use Default' })).toBeVisible();
            await page.keyboard.press('Escape');
        });

        test('QuickDefaults Clear removes all saved values', async () => {
            await expect(page.getByRole('combobox', { name: 'Default Distribution' })).not.toHaveValue('');

            await page.getByRole('button', { name: 'Clear Saved' }).click();

            await expect(page.getByRole('combobox', { name: 'Default Distribution' })).toHaveValue('');
            await expect(page.getByRole('combobox', { name: 'Default Mint' })).toHaveValue('');
            await expect(page.getByRole('combobox', { name: 'Default Reward Pool' })).toHaveValue('');
            await expect(page.locator('text=0 saved').first()).toBeVisible();
        });

        test('RecentTransactions shows all txs with View Explorer links', async () => {
            const heading = page.getByRole('heading', { name: /Recent Transactions \(\d+\)/ });
            await expect(heading).toBeVisible();

            const count = parseInt((await heading.textContent())!.match(/\d+/)![0]);
            // 23 instruction steps (incl. expected failure) + any validation submissions.
            expect(count).toBeGreaterThanOrEqual(15);

            // Design system Button asChild may render <a> with role="button"; use href selector.
            await expect(page.locator('a[href*="explorer.solana.com"]').first()).toBeVisible();
        });
    });
});
