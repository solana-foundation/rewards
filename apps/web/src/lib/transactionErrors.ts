import {
    isSolanaError,
    SOLANA_ERROR__INSTRUCTION_ERROR__CUSTOM,
    SOLANA_ERROR__JSON_RPC__SERVER_ERROR_SEND_TRANSACTION_PREFLIGHT_FAILURE,
    unwrapSimulationError,
} from '@solana/kit';
import {
    REWARDS_PROGRAM_ERROR__CLAIMANT_ALREADY_REVOKED,
    REWARDS_PROGRAM_ERROR__CLAIMED_AMOUNT_DECREASED,
    REWARDS_PROGRAM_ERROR__CLAIM_NOT_FULLY_VESTED,
    REWARDS_PROGRAM_ERROR__CLAWBACK_NOT_REACHED,
    REWARDS_PROGRAM_ERROR__DISTRIBUTION_NOT_REVOCABLE,
    REWARDS_PROGRAM_ERROR__DISTRIBUTION_PERMANENTLY_CLOSED,
    REWARDS_PROGRAM_ERROR__EXCEEDS_CLAIMABLE_AMOUNT,
    REWARDS_PROGRAM_ERROR__INSUFFICIENT_FUNDS,
    REWARDS_PROGRAM_ERROR__INVALID_ACCOUNT_DATA,
    REWARDS_PROGRAM_ERROR__INVALID_AMOUNT,
    REWARDS_PROGRAM_ERROR__INVALID_CLIFF_TIMESTAMP,
    REWARDS_PROGRAM_ERROR__INVALID_EVENT_AUTHORITY,
    REWARDS_PROGRAM_ERROR__INVALID_MERKLE_PROOF,
    REWARDS_PROGRAM_ERROR__INVALID_REVOKE_MODE,
    REWARDS_PROGRAM_ERROR__INVALID_SCHEDULE_TYPE,
    REWARDS_PROGRAM_ERROR__INVALID_TIME_WINDOW,
    REWARDS_PROGRAM_ERROR__MATH_OVERFLOW,
    REWARDS_PROGRAM_ERROR__NOTHING_TO_CLAIM,
    REWARDS_PROGRAM_ERROR__RENT_CALCULATION_FAILED,
    REWARDS_PROGRAM_ERROR__UNAUTHORIZED_AUTHORITY,
    REWARDS_PROGRAM_ERROR__UNAUTHORIZED_RECIPIENT,
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
    [REWARDS_PROGRAM_ERROR__DISTRIBUTION_PERMANENTLY_CLOSED]: 'Distribution has been permanently closed',
};

const FALLBACK_TX_FAILED_MESSAGE = 'Transaction failed';
const MAX_LOG_LINES = 12;

export interface TransactionErrorDetails {
    readonly logs: readonly string[];
    readonly message: string;
}

function isRecord(value: unknown): value is Record<string, unknown> {
    return typeof value === 'object' && value !== null;
}

function getErrorMessage(error: unknown): string {
    if (typeof error === 'string') return error;
    if (error instanceof Error) return error.message;
    if (isRecord(error) && typeof error.message === 'string') return error.message;
    return '';
}

function getErrorCause(error: unknown): unknown {
    if (error instanceof Error) return error.cause;
    if (isRecord(error)) return error.cause;
    return undefined;
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

function parseInstructionErrorCode(value: unknown): number | null {
    if (!Array.isArray(value) || value.length < 2) return null;

    const instructionError = value[1];
    if (isRecord(instructionError) && typeof instructionError.Custom === 'number') return instructionError.Custom;
    return null;
}

function parseCustomProgramCode(error: unknown, visited = new Set<object>()): number | null {
    if (isRecord(error)) {
        if (visited.has(error)) return null;
        visited.add(error);
    }

    const simulationCause = unwrapSimulationError(error);
    if (simulationCause !== error) {
        const simulationCode = parseCustomProgramCode(simulationCause, visited);
        if (simulationCode !== null) return simulationCode;
    }

    if (isSolanaError(error, SOLANA_ERROR__INSTRUCTION_ERROR__CUSTOM)) {
        return error.context.code;
    }

    if (isRecord(error)) {
        const context = error.context;
        if (isRecord(context) && typeof context.code === 'number') return context.code;

        const directInstructionErrorCode = parseInstructionErrorCode(error.InstructionError);
        if (directInstructionErrorCode !== null) return directInstructionErrorCode;

        const err = error.err;
        if (isRecord(err)) {
            const errInstructionErrorCode = parseInstructionErrorCode(err.InstructionError);
            if (errInstructionErrorCode !== null) return errInstructionErrorCode;
        }

        const data = error.data;
        if (isRecord(data)) {
            const dataInstructionErrorCode = parseCustomProgramCode(data, visited);
            if (dataInstructionErrorCode !== null) return dataInstructionErrorCode;
        }
    }

    const message = getErrorMessage(error);
    const parsedMessageCode = message ? parseCustomProgramCodeFromString(message) : null;
    if (parsedMessageCode !== null) return parsedMessageCode;

    const cause = getErrorCause(error);
    return cause === undefined ? null : parseCustomProgramCode(cause, visited);
}

function normalizeLogs(value: unknown): readonly string[] {
    if (!Array.isArray(value)) return [];
    return value.filter((line): line is string => typeof line === 'string' && line.trim().length > 0);
}

function collectLogs(error: unknown, visited = new Set<object>()): readonly string[] {
    const logs: string[] = [];

    if (isRecord(error)) {
        if (visited.has(error)) return [];
        visited.add(error);

        if (isSolanaError(error, SOLANA_ERROR__JSON_RPC__SERVER_ERROR_SEND_TRANSACTION_PREFLIGHT_FAILURE)) {
            logs.push(...normalizeLogs(error.context.logs));
        }

        logs.push(...normalizeLogs(error.logs));

        const context = error.context;
        if (isRecord(context)) logs.push(...normalizeLogs(context.logs));

        const data = error.data;
        if (isRecord(data)) logs.push(...normalizeLogs(data.logs));
    }

    const simulationCause = unwrapSimulationError(error);
    if (simulationCause !== error) logs.push(...collectLogs(simulationCause, visited));

    const cause = getErrorCause(error);
    if (cause !== undefined) logs.push(...collectLogs(cause, visited));

    return [...new Set(logs)];
}

function isPreflightFailure(error: unknown, visited = new Set<object>()): boolean {
    if (isRecord(error)) {
        if (visited.has(error)) return false;
        visited.add(error);
    }

    if (isSolanaError(error, SOLANA_ERROR__JSON_RPC__SERVER_ERROR_SEND_TRANSACTION_PREFLIGHT_FAILURE)) {
        return true;
    }

    const cause = getErrorCause(error);
    return cause === undefined ? false : isPreflightFailure(cause, visited);
}

export function getTransactionErrorDetails(error: unknown): TransactionErrorDetails {
    const message = getErrorMessage(error).trim();
    const code = parseCustomProgramCode(error);
    const rewardsMessage = code !== null ? ERROR_MESSAGES[code] : null;

    if (rewardsMessage) {
        return {
            logs: collectLogs(error),
            message: `${FALLBACK_TX_FAILED_MESSAGE}: ${rewardsMessage}`,
        };
    }

    if (/user rejected|rejected the request|declined|cancelled/i.test(message)) {
        return {
            logs: [],
            message: 'Transaction was rejected in wallet',
        };
    }

    if (/request.*pending|already pending/i.test(message)) {
        return {
            logs: [],
            message: `${FALLBACK_TX_FAILED_MESSAGE}: request is already pending in your wallet`,
        };
    }

    if (isPreflightFailure(error)) {
        return {
            logs: collectLogs(error),
            message: `${FALLBACK_TX_FAILED_MESSAGE}: transaction simulation failed`,
        };
    }

    if (
        message === FALLBACK_TX_FAILED_MESSAGE ||
        message.startsWith(`${FALLBACK_TX_FAILED_MESSAGE}:`) ||
        message === 'Transaction was rejected in wallet'
    ) {
        return {
            logs: collectLogs(error),
            message,
        };
    }

    return {
        logs: collectLogs(error),
        message: message ? `${FALLBACK_TX_FAILED_MESSAGE}: ${message}` : FALLBACK_TX_FAILED_MESSAGE,
    };
}

export function formatTransactionError(error: unknown): string {
    return getTransactionErrorDetails(error).message;
}

export function formatTransactionErrorWithLogs(error: unknown): string {
    const { logs, message } = getTransactionErrorDetails(error);
    if (logs.length === 0) return message;

    const visibleLogs = logs.slice(-MAX_LOG_LINES);
    const omittedCount = logs.length - visibleLogs.length;
    const omittedLine = omittedCount > 0 ? [`... ${omittedCount} earlier log lines omitted`] : [];

    return [message, '', 'Logs:', ...omittedLine, ...visibleLogs].join('\n');
}
