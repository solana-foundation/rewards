'use client';

import {
    REWARDS_PROGRAM_ERROR__BALANCE_SOURCE_MISMATCH,
    REWARDS_PROGRAM_ERROR__CLAIMANT_ALREADY_REVOKED,
    REWARDS_PROGRAM_ERROR__CLAIMED_AMOUNT_DECREASED,
    REWARDS_PROGRAM_ERROR__CLAIM_NOT_FULLY_VESTED,
    REWARDS_PROGRAM_ERROR__CLAWBACK_NOT_REACHED,
    REWARDS_PROGRAM_ERROR__CONTINUOUS_MERKLE_MODE_ENABLED,
    REWARDS_PROGRAM_ERROR__DISTRIBUTION_AMOUNT_TOO_SMALL,
    REWARDS_PROGRAM_ERROR__DISTRIBUTION_NOT_REVOCABLE,
    REWARDS_PROGRAM_ERROR__DISTRIBUTION_PERMANENTLY_CLOSED,
    REWARDS_PROGRAM_ERROR__EXCEEDS_CLAIMABLE_AMOUNT,
    REWARDS_PROGRAM_ERROR__INSUFFICIENT_FUNDS,
    REWARDS_PROGRAM_ERROR__INSUFFICIENT_POINTS_BALANCE,
    REWARDS_PROGRAM_ERROR__INVALID_ACCOUNT_DATA,
    REWARDS_PROGRAM_ERROR__INVALID_AMOUNT,
    REWARDS_PROGRAM_ERROR__INVALID_BALANCE_SOURCE,
    REWARDS_PROGRAM_ERROR__INVALID_CLIFF_TIMESTAMP,
    REWARDS_PROGRAM_ERROR__INVALID_EVENT_AUTHORITY,
    REWARDS_PROGRAM_ERROR__INVALID_MERKLE_PROOF,
    REWARDS_PROGRAM_ERROR__INVALID_MERKLE_ROOT_VERSION,
    REWARDS_PROGRAM_ERROR__INVALID_REVOKE_MODE,
    REWARDS_PROGRAM_ERROR__INVALID_SCHEDULE_TYPE,
    REWARDS_PROGRAM_ERROR__INVALID_TIMESTAMP,
    REWARDS_PROGRAM_ERROR__INVALID_TIME_WINDOW,
    REWARDS_PROGRAM_ERROR__MATH_OVERFLOW,
    REWARDS_PROGRAM_ERROR__MERKLE_ROOT_NOT_SET,
    REWARDS_PROGRAM_ERROR__MERKLE_ROOT_VERSION_MISMATCH,
    REWARDS_PROGRAM_ERROR__NO_OPTED_IN_USERS,
    REWARDS_PROGRAM_ERROR__NOTHING_TO_CLAIM,
    REWARDS_PROGRAM_ERROR__POINTS_BALANCE_NOT_ZERO,
    REWARDS_PROGRAM_ERROR__POINTS_MAX_SUPPLY_EXCEEDED,
    REWARDS_PROGRAM_ERROR__POINTS_NOTHING_TO_REVOKE,
    REWARDS_PROGRAM_ERROR__POINTS_NOT_REVOCABLE,
    REWARDS_PROGRAM_ERROR__POINTS_SELF_TRANSFER_NOT_ALLOWED,
    REWARDS_PROGRAM_ERROR__POINTS_TRANSFERS_DISABLED,
    REWARDS_PROGRAM_ERROR__RENT_CALCULATION_FAILED,
    REWARDS_PROGRAM_ERROR__REWARD_MINT_MISMATCH,
    REWARDS_PROGRAM_ERROR__TRACKED_MINT_MISMATCH,
    REWARDS_PROGRAM_ERROR__TRANSFER_HOOK_MINT_UNSUPPORTED,
    REWARDS_PROGRAM_ERROR__UNAUTHORIZED_AUTHORITY,
    REWARDS_PROGRAM_ERROR__UNAUTHORIZED_RECIPIENT,
    REWARDS_PROGRAM_ERROR__USER_ALREADY_OPTED_IN,
    REWARDS_PROGRAM_ERROR__USER_ALREADY_REVOKED,
    REWARDS_PROGRAM_ERROR__USER_NOT_OPTED_IN,
    REWARDS_PROGRAM_ERROR__USER_REVOKED,
} from '@solana/rewards';

const ERROR_MESSAGES: Record<number, string> = {
    [REWARDS_PROGRAM_ERROR__INVALID_AMOUNT]: 'Invalid amount specified',
    [REWARDS_PROGRAM_ERROR__INVALID_TIME_WINDOW]: 'Invalid time window configuration',
    [REWARDS_PROGRAM_ERROR__INVALID_SCHEDULE_TYPE]: 'Invalid schedule type',
    [REWARDS_PROGRAM_ERROR__UNAUTHORIZED_AUTHORITY]: 'Unauthorized authority',
    [REWARDS_PROGRAM_ERROR__UNAUTHORIZED_RECIPIENT]: 'Unauthorized recipient',
    [REWARDS_PROGRAM_ERROR__INSUFFICIENT_FUNDS]: 'Insufficient funds in distribution',
    [REWARDS_PROGRAM_ERROR__NOTHING_TO_CLAIM]: 'Nothing available to claim',
    [REWARDS_PROGRAM_ERROR__MATH_OVERFLOW]: 'Math overflow occurred',
    [REWARDS_PROGRAM_ERROR__INVALID_ACCOUNT_DATA]: 'Invalid account data',
    [REWARDS_PROGRAM_ERROR__INVALID_EVENT_AUTHORITY]: 'Event authority PDA is invalid',
    [REWARDS_PROGRAM_ERROR__RENT_CALCULATION_FAILED]: 'Rent calculation failed',
    [REWARDS_PROGRAM_ERROR__EXCEEDS_CLAIMABLE_AMOUNT]: 'Requested claim amount exceeds available balance',
    [REWARDS_PROGRAM_ERROR__INVALID_MERKLE_PROOF]: 'Invalid merkle proof',
    [REWARDS_PROGRAM_ERROR__CLAWBACK_NOT_REACHED]: 'Clawback timestamp not yet reached',
    [REWARDS_PROGRAM_ERROR__CLAIM_NOT_FULLY_VESTED]: 'Claim has not been fully vested',
    [REWARDS_PROGRAM_ERROR__INVALID_CLIFF_TIMESTAMP]: 'Invalid cliff timestamp',
    [REWARDS_PROGRAM_ERROR__CLAIMED_AMOUNT_DECREASED]: 'Claimed amount cannot decrease',
    [REWARDS_PROGRAM_ERROR__DISTRIBUTION_NOT_REVOCABLE]: 'Distribution is not revocable',
    [REWARDS_PROGRAM_ERROR__INVALID_REVOKE_MODE]: 'Invalid revoke mode',
    [REWARDS_PROGRAM_ERROR__CLAIMANT_ALREADY_REVOKED]: 'Claimant has already been revoked',
    [REWARDS_PROGRAM_ERROR__TRANSFER_HOOK_MINT_UNSUPPORTED]: 'Transfer hook mints are not supported',
    [REWARDS_PROGRAM_ERROR__DISTRIBUTION_PERMANENTLY_CLOSED]: 'Distribution has been permanently closed',
    [REWARDS_PROGRAM_ERROR__NO_OPTED_IN_USERS]: 'No users opted in to receive rewards',
    [REWARDS_PROGRAM_ERROR__USER_ALREADY_OPTED_IN]: 'User is already opted in',
    [REWARDS_PROGRAM_ERROR__USER_NOT_OPTED_IN]: 'User is not opted in',
    [REWARDS_PROGRAM_ERROR__DISTRIBUTION_AMOUNT_TOO_SMALL]: 'Distribution amount too small for opted-in supply',
    [REWARDS_PROGRAM_ERROR__TRACKED_MINT_MISMATCH]: 'Tracked mint does not match pool',
    [REWARDS_PROGRAM_ERROR__REWARD_MINT_MISMATCH]: 'Reward mint does not match pool',
    [REWARDS_PROGRAM_ERROR__INVALID_BALANCE_SOURCE]: 'Invalid balance source mode',
    [REWARDS_PROGRAM_ERROR__BALANCE_SOURCE_MISMATCH]: "Instruction not allowed for this pool's balance source mode",
    [REWARDS_PROGRAM_ERROR__USER_REVOKED]: 'User has been revoked from this reward pool',
    [REWARDS_PROGRAM_ERROR__USER_ALREADY_REVOKED]: 'User has already been revoked from this reward pool',
    [REWARDS_PROGRAM_ERROR__INVALID_TIMESTAMP]: 'Invalid timestamp value',
    [REWARDS_PROGRAM_ERROR__INVALID_MERKLE_ROOT_VERSION]: 'Invalid merkle root version value',
    [REWARDS_PROGRAM_ERROR__MERKLE_ROOT_NOT_SET]: 'Merkle root is not configured for this pool',
    [REWARDS_PROGRAM_ERROR__MERKLE_ROOT_VERSION_MISMATCH]:
        'Merkle proof root version does not match the pool root version',
    [REWARDS_PROGRAM_ERROR__CONTINUOUS_MERKLE_MODE_ENABLED]: 'This pool is configured for merkle claims',
    [REWARDS_PROGRAM_ERROR__POINTS_MAX_SUPPLY_EXCEEDED]: 'Points max supply exceeded',
    [REWARDS_PROGRAM_ERROR__INSUFFICIENT_POINTS_BALANCE]: 'Insufficient points balance',
    [REWARDS_PROGRAM_ERROR__POINTS_TRANSFERS_DISABLED]: 'Points transfers not allowed for this config',
    [REWARDS_PROGRAM_ERROR__POINTS_BALANCE_NOT_ZERO]: 'Points account balance not zero',
    [REWARDS_PROGRAM_ERROR__POINTS_NOT_REVOCABLE]: 'Points config is not revocable',
    [REWARDS_PROGRAM_ERROR__POINTS_SELF_TRANSFER_NOT_ALLOWED]: 'Cannot transfer points to the same user',
    [REWARDS_PROGRAM_ERROR__POINTS_NOTHING_TO_REVOKE]: 'Nothing to revoke - user has zero balance',
};

const FALLBACK_TX_FAILED_MESSAGE = 'Transaction failed';

function getErrorMessage(error: unknown): string {
    if (error instanceof Error) return error.message;
    if (typeof error === 'string') return error;
    return '';
}

function parseCustomProgramCodeFromString(message: string): number | null {
    const customErrorMatch = message.match(/custom program error:\s*(#\d+|0x[0-9a-fA-F]+|\d+)/i);
    if (!customErrorMatch) return null;

    const value = customErrorMatch[1].trim();
    if (value.startsWith('#')) {
        const parsed = Number.parseInt(value.slice(1), 10);
        return Number.isNaN(parsed) ? null : parsed;
    }
    if (value.toLowerCase().startsWith('0x')) {
        const parsed = Number.parseInt(value.slice(2), 16);
        return Number.isNaN(parsed) ? null : parsed;
    }
    const parsed = Number.parseInt(value, 10);
    return Number.isNaN(parsed) ? null : parsed;
}

function parseCustomProgramCode(error: unknown): number | null {
    if (error && typeof error === 'object') {
        const withContext = error as { context?: { code?: unknown } };
        if (typeof withContext.context?.code === 'number') {
            return withContext.context.code;
        }
    }

    const message = getErrorMessage(error);
    if (!message) return null;
    return parseCustomProgramCodeFromString(message);
}

export function formatTransactionError(error: unknown): string {
    const message = getErrorMessage(error);

    if (
        message === FALLBACK_TX_FAILED_MESSAGE ||
        message.startsWith(`${FALLBACK_TX_FAILED_MESSAGE}:`) ||
        message === 'Transaction was rejected in wallet'
    ) {
        return message;
    }

    const code = parseCustomProgramCode(error);
    const rewardsMessage = code !== null ? ERROR_MESSAGES[code] : null;
    if (rewardsMessage) {
        return `${FALLBACK_TX_FAILED_MESSAGE}: ${rewardsMessage}`;
    }

    if (message.includes('-32002')) {
        return `${FALLBACK_TX_FAILED_MESSAGE}: request is already pending in your wallet`;
    }

    if (/user rejected|rejected the request|declined|cancelled/i.test(message)) {
        return 'Transaction was rejected in wallet';
    }

    return FALLBACK_TX_FAILED_MESSAGE;
}
