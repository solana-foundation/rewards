'use client';

import { createContext, useCallback, useContext, useEffect, useMemo, useState } from 'react';

const STORAGE_KEY = 'rewards-ui-saved-values-v1';
const MAX_SAVED_VALUES = 25;

interface SavedValuesState {
    defaultDistribution: string;
    defaultRewardPool: string;
    defaultMint: string;
    defaultTrackedMint: string;
    defaultRewardMint: string;
    distributions: string[];
    rewardPools: string[];
    mints: string[];
    trackedMints: string[];
    rewardMints: string[];
}

const INITIAL_STATE: SavedValuesState = {
    defaultDistribution: '',
    defaultRewardPool: '',
    defaultMint: '',
    defaultTrackedMint: '',
    defaultRewardMint: '',
    distributions: [],
    rewardPools: [],
    mints: [],
    trackedMints: [],
    rewardMints: [],
};

interface SavedValuesContextType extends SavedValuesState {
    setDefaultDistribution: (value: string) => void;
    setDefaultRewardPool: (value: string) => void;
    setDefaultMint: (value: string) => void;
    setDefaultTrackedMint: (value: string) => void;
    setDefaultRewardMint: (value: string) => void;
    rememberDistribution: (value: string) => void;
    rememberRewardPool: (value: string) => void;
    rememberMint: (value: string) => void;
    rememberTrackedMint: (value: string) => void;
    rememberRewardMint: (value: string) => void;
    clearSavedValues: () => void;
}

const SavedValuesContext = createContext<SavedValuesContextType | null>(null);

function normalizeValue(value: string) {
    return value.trim();
}

function addUnique(values: string[], value: string): string[] {
    const normalized = normalizeValue(value);
    if (!normalized) return values;
    return [normalized, ...values.filter(v => v !== normalized)].slice(0, MAX_SAVED_VALUES);
}

function isStringArray(value: unknown): value is string[] {
    return Array.isArray(value) && value.every(item => typeof item === 'string');
}

function isSavedValuesState(value: unknown): value is SavedValuesState {
    if (!value || typeof value !== 'object') return false;
    const candidate = value as Record<string, unknown>;
    return (
        typeof candidate.defaultDistribution === 'string' &&
        typeof candidate.defaultRewardPool === 'string' &&
        typeof candidate.defaultMint === 'string' &&
        typeof candidate.defaultTrackedMint === 'string' &&
        typeof candidate.defaultRewardMint === 'string' &&
        isStringArray(candidate.distributions) &&
        isStringArray(candidate.rewardPools) &&
        isStringArray(candidate.mints) &&
        isStringArray(candidate.trackedMints) &&
        isStringArray(candidate.rewardMints)
    );
}

function readFromStorage(): SavedValuesState {
    try {
        const raw = window.localStorage.getItem(STORAGE_KEY);
        if (!raw) return INITIAL_STATE;
        const parsed: unknown = JSON.parse(raw);
        if (!isSavedValuesState(parsed)) return INITIAL_STATE;
        return {
            defaultDistribution: normalizeValue(parsed.defaultDistribution),
            defaultRewardPool: normalizeValue(parsed.defaultRewardPool),
            defaultMint: normalizeValue(parsed.defaultMint),
            defaultTrackedMint: normalizeValue(parsed.defaultTrackedMint),
            defaultRewardMint: normalizeValue(parsed.defaultRewardMint),
            distributions: parsed.distributions.map(normalizeValue).filter(Boolean).slice(0, MAX_SAVED_VALUES),
            rewardPools: parsed.rewardPools.map(normalizeValue).filter(Boolean).slice(0, MAX_SAVED_VALUES),
            mints: parsed.mints.map(normalizeValue).filter(Boolean).slice(0, MAX_SAVED_VALUES),
            trackedMints: parsed.trackedMints.map(normalizeValue).filter(Boolean).slice(0, MAX_SAVED_VALUES),
            rewardMints: parsed.rewardMints.map(normalizeValue).filter(Boolean).slice(0, MAX_SAVED_VALUES),
        };
    } catch {
        return INITIAL_STATE;
    }
}

export function SavedValuesProvider({ children }: { children: React.ReactNode }) {
    const [state, setState] = useState<SavedValuesState>(INITIAL_STATE);
    const [hydrated, setHydrated] = useState(false);

    useEffect(() => {
        setState(readFromStorage());
        setHydrated(true);
    }, []);

    useEffect(() => {
        if (!hydrated) return;
        window.localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
    }, [state, hydrated]);

    const setDefaultDistribution = useCallback((value: string) => {
        setState(current => ({ ...current, defaultDistribution: normalizeValue(value) }));
    }, []);

    const setDefaultRewardPool = useCallback((value: string) => {
        setState(current => ({ ...current, defaultRewardPool: normalizeValue(value) }));
    }, []);

    const setDefaultMint = useCallback((value: string) => {
        setState(current => ({ ...current, defaultMint: normalizeValue(value) }));
    }, []);

    const setDefaultTrackedMint = useCallback((value: string) => {
        setState(current => ({ ...current, defaultTrackedMint: normalizeValue(value) }));
    }, []);

    const setDefaultRewardMint = useCallback((value: string) => {
        setState(current => ({ ...current, defaultRewardMint: normalizeValue(value) }));
    }, []);

    const rememberDistribution = useCallback((value: string) => {
        setState(current => {
            const normalized = normalizeValue(value);
            if (!normalized) return current;
            return {
                ...current,
                defaultDistribution: normalized,
                distributions: addUnique(current.distributions, normalized),
            };
        });
    }, []);

    const rememberRewardPool = useCallback((value: string) => {
        setState(current => {
            const normalized = normalizeValue(value);
            if (!normalized) return current;
            return {
                ...current,
                defaultRewardPool: normalized,
                rewardPools: addUnique(current.rewardPools, normalized),
            };
        });
    }, []);

    const rememberMint = useCallback((value: string) => {
        setState(current => {
            const normalized = normalizeValue(value);
            if (!normalized) return current;
            return {
                ...current,
                defaultMint: normalized,
                mints: addUnique(current.mints, normalized),
            };
        });
    }, []);

    const rememberTrackedMint = useCallback((value: string) => {
        setState(current => {
            const normalized = normalizeValue(value);
            if (!normalized) return current;
            return {
                ...current,
                defaultTrackedMint: normalized,
                trackedMints: addUnique(current.trackedMints, normalized),
            };
        });
    }, []);

    const rememberRewardMint = useCallback((value: string) => {
        setState(current => {
            const normalized = normalizeValue(value);
            if (!normalized) return current;
            return {
                ...current,
                defaultRewardMint: normalized,
                rewardMints: addUnique(current.rewardMints, normalized),
            };
        });
    }, []);

    const clearSavedValues = useCallback(() => {
        setState(INITIAL_STATE);
    }, []);

    const contextValue = useMemo<SavedValuesContextType>(
        () => ({
            ...state,
            setDefaultDistribution,
            setDefaultRewardPool,
            setDefaultMint,
            setDefaultTrackedMint,
            setDefaultRewardMint,
            rememberDistribution,
            rememberRewardPool,
            rememberMint,
            rememberTrackedMint,
            rememberRewardMint,
            clearSavedValues,
        }),
        [
            state,
            setDefaultDistribution,
            setDefaultRewardPool,
            setDefaultMint,
            setDefaultTrackedMint,
            setDefaultRewardMint,
            rememberDistribution,
            rememberRewardPool,
            rememberMint,
            rememberTrackedMint,
            rememberRewardMint,
            clearSavedValues,
        ],
    );

    return <SavedValuesContext.Provider value={contextValue}>{children}</SavedValuesContext.Provider>;
}

export function useSavedValues() {
    const context = useContext(SavedValuesContext);
    if (!context) {
        throw new Error('useSavedValues must be used inside SavedValuesProvider');
    }
    return context;
}
