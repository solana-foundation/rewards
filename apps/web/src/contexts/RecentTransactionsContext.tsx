import { createContext, useCallback, useContext, useEffect, useMemo, useState } from 'react';
import { formatTransactionError } from '@/lib/transactionErrors';

const STORAGE_KEY = 'rewards-ui-recent-transactions-v1';
const MAX_RECENT_TRANSACTIONS = 20;

export interface RecentTransactionValues {
    distribution?: string;
    mint?: string;
    recipient?: string;
    claimant?: string;
    user?: string;
    amount?: string;
    totalAmount?: string;
    rootVersion?: string;
}

const RECENT_VALUE_KEYS = [
    'distribution',
    'mint',
    'recipient',
    'claimant',
    'user',
    'amount',
    'totalAmount',
    'rootVersion',
] as const satisfies readonly (keyof RecentTransactionValues)[];

export interface RecentTransaction {
    id: string;
    signature: string | null;
    action: string;
    timestamp: number;
    status: 'success' | 'failed';
    error?: string;
    values?: RecentTransactionValues;
}

interface RecentTransactionsContextType {
    recentTransactions: RecentTransaction[];
    addRecentTransaction: (transaction: RecentTransaction) => void;
    clearRecentTransactions: () => void;
}

const RecentTransactionsContext = createContext<RecentTransactionsContextType | null>(null);

function normalizeValues(values?: RecentTransactionValues): RecentTransactionValues | undefined {
    if (!values) return undefined;
    const valueRecord = values as Record<keyof RecentTransactionValues, string | undefined>;
    const normalizedEntries = RECENT_VALUE_KEYS.map(key => [key, valueRecord[key]?.trim() ?? ''] as const).filter(
        ([, value]) => value.length > 0,
    );
    if (normalizedEntries.length === 0) return undefined;
    return Object.fromEntries(normalizedEntries) as RecentTransactionValues;
}

function isRecord(value: unknown): value is Record<string, unknown> {
    return typeof value === 'object' && value !== null;
}

function readStoredTransactions(): RecentTransaction[] {
    try {
        const raw = window.localStorage.getItem(STORAGE_KEY);
        if (!raw) return [];
        const parsed: unknown = JSON.parse(raw);
        if (!Array.isArray(parsed)) return [];
        return parsed
            .filter(isRecord)
            .map(item => {
                const signatureValue = item.signature;
                const actionValue = item.action;
                const timestampValue = item.timestamp;
                const statusValue = item.status;
                const errorValue = item.error;
                const valuesValue = item.values;

                const fallbackId = `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
                const id =
                    typeof item.id === 'string'
                        ? item.id
                        : typeof signatureValue === 'string'
                          ? signatureValue
                          : fallbackId;
                const action = typeof actionValue === 'string' ? actionValue : 'Transaction';
                const error = typeof errorValue === 'string' ? formatTransactionError(errorValue) : undefined;
                const timestamp =
                    typeof timestampValue === 'number'
                        ? timestampValue
                        : typeof timestampValue === 'string'
                          ? Number(timestampValue)
                          : Date.now();

                return {
                    action,
                    error,
                    id,
                    signature: typeof signatureValue === 'string' ? signatureValue : null,
                    status: statusValue === 'failed' ? ('failed' as const) : ('success' as const),
                    timestamp,
                    values: isRecord(valuesValue) ? normalizeValues(valuesValue as RecentTransactionValues) : undefined,
                };
            })
            .slice(0, MAX_RECENT_TRANSACTIONS);
    } catch {
        return [];
    }
}

export function RecentTransactionsProvider({ children }: { children: React.ReactNode }) {
    const [recentTransactions, setRecentTransactions] = useState<RecentTransaction[]>([]);
    const [hydrated, setHydrated] = useState(false);

    useEffect(() => {
        setRecentTransactions(readStoredTransactions());
        setHydrated(true);
    }, []);

    useEffect(() => {
        if (!hydrated) return;
        window.localStorage.setItem(STORAGE_KEY, JSON.stringify(recentTransactions));
    }, [hydrated, recentTransactions]);

    const addRecentTransaction = useCallback((transaction: RecentTransaction) => {
        setRecentTransactions(current => {
            const normalizedSignature = transaction.signature?.trim() || null;
            const normalized: RecentTransaction = {
                ...transaction,
                id: transaction.id.trim() || `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
                signature: normalizedSignature,
                action: transaction.action.trim() || 'Transaction',
                status: transaction.status,
                error: transaction.error?.trim() ? formatTransactionError(transaction.error) : undefined,
                values: normalizeValues(transaction.values),
            };

            const deduped = current.filter(item =>
                normalized.signature ? item.signature !== normalized.signature : item.id !== normalized.id,
            );
            return [normalized, ...deduped].slice(0, MAX_RECENT_TRANSACTIONS);
        });
    }, []);

    const clearRecentTransactions = useCallback(() => {
        setRecentTransactions([]);
    }, []);

    const value = useMemo(
        () => ({
            recentTransactions,
            addRecentTransaction,
            clearRecentTransactions,
        }),
        [recentTransactions, addRecentTransaction, clearRecentTransactions],
    );

    return <RecentTransactionsContext.Provider value={value}>{children}</RecentTransactionsContext.Provider>;
}

export function useRecentTransactions() {
    const context = useContext(RecentTransactionsContext);
    if (!context) {
        throw new Error('useRecentTransactions must be used inside RecentTransactionsProvider');
    }
    return context;
}
