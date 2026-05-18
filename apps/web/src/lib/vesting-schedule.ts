import type { VestingSchedule, VestingScheduleArgs } from '@solana/rewards';

import { parseBigIntValue, validateInteger } from '@/lib/validation';

export interface VestingScheduleState {
    kind: 'Immediate' | 'Linear' | 'Cliff' | 'CliffLinear';
    startTs: string;
    cliffTs: string;
    endTs: string;
}

export const INITIAL_VESTING_SCHEDULE: VestingScheduleState = {
    kind: 'Immediate',
    startTs: '',
    cliffTs: '',
    endTs: '',
};

export function isVestingScheduleState(value: unknown): value is VestingScheduleState {
    if (!value || typeof value !== 'object') return false;
    const candidate = value as Record<string, unknown>;
    return (
        (candidate.kind === 'Immediate' ||
            candidate.kind === 'Linear' ||
            candidate.kind === 'Cliff' ||
            candidate.kind === 'CliffLinear') &&
        typeof candidate.startTs === 'string' &&
        typeof candidate.cliffTs === 'string' &&
        typeof candidate.endTs === 'string'
    );
}

export function buildVestingSchedule(
    value: VestingScheduleState,
): { ok: true; value: VestingScheduleArgs } | { ok: false; error: string } {
    if (value.kind === 'Immediate') {
        return { ok: true, value: { __kind: 'Immediate' } };
    }

    if (value.kind === 'Linear') {
        const startErr = validateInteger(value.startTs, 'Start timestamp');
        if (startErr) return { ok: false, error: startErr };
        const endErr = validateInteger(value.endTs, 'End timestamp');
        if (endErr) return { ok: false, error: endErr };
        return {
            ok: true,
            value: {
                __kind: 'Linear',
                startTs: parseBigIntValue(value.startTs),
                endTs: parseBigIntValue(value.endTs),
            },
        };
    }

    if (value.kind === 'Cliff') {
        const cliffErr = validateInteger(value.cliffTs, 'Cliff timestamp');
        if (cliffErr) return { ok: false, error: cliffErr };
        return {
            ok: true,
            value: {
                __kind: 'Cliff',
                cliffTs: parseBigIntValue(value.cliffTs),
            },
        };
    }

    const startErr = validateInteger(value.startTs, 'Start timestamp');
    if (startErr) return { ok: false, error: startErr };
    const cliffErr = validateInteger(value.cliffTs, 'Cliff timestamp');
    if (cliffErr) return { ok: false, error: cliffErr };
    const endErr = validateInteger(value.endTs, 'End timestamp');
    if (endErr) return { ok: false, error: endErr };

    return {
        ok: true,
        value: {
            __kind: 'CliffLinear',
            startTs: parseBigIntValue(value.startTs),
            cliffTs: parseBigIntValue(value.cliffTs),
            endTs: parseBigIntValue(value.endTs),
        },
    };
}

export function vestingScheduleToState(schedule: VestingSchedule | VestingScheduleArgs): VestingScheduleState {
    if (schedule.__kind === 'Immediate') return INITIAL_VESTING_SCHEDULE;
    if (schedule.__kind === 'Linear') {
        return {
            kind: 'Linear',
            startTs: String(schedule.startTs),
            cliffTs: '',
            endTs: String(schedule.endTs),
        };
    }
    if (schedule.__kind === 'Cliff') {
        return {
            kind: 'Cliff',
            startTs: '',
            cliffTs: String(schedule.cliffTs),
            endTs: '',
        };
    }
    return {
        kind: 'CliffLinear',
        startTs: String(schedule.startTs),
        cliffTs: String(schedule.cliffTs),
        endTs: String(schedule.endTs),
    };
}
