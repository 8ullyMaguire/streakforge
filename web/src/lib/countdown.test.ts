import { describe, it, expect } from 'vitest';
import { remainingSecs, formatCountdown, formatUntil } from './countdown';

describe('remainingSecs', () => {
	it('returns the server count when no time has elapsed', () => {
		expect(remainingSecs(3600, 0)).toBe(3600);
	});

	it('decrements as time passes', () => {
		expect(remainingSecs(3600, 1000)).toBe(3599);
		expect(remainingSecs(3600, 60_000)).toBe(3540);
	});

	it('clamps at zero instead of going negative', () => {
		expect(remainingSecs(10, 60_000)).toBe(0);
		expect(remainingSecs(0, 5000)).toBe(0);
		expect(remainingSecs(-5, 0)).toBe(0);
	});

	it('rounds partial seconds down (a countdown must not jump early)', () => {
		expect(remainingSecs(100, 999)).toBe(100);
		expect(remainingSecs(100, 1000)).toBe(99);
	});
});

describe('formatCountdown', () => {
	it('formats as HH:MM:SS zero-padded', () => {
		expect(formatCountdown(0)).toBe('00:00:00');
		expect(formatCountdown(61)).toBe('00:01:01');
		expect(formatCountdown(3661)).toBe('01:01:01');
	});

	it('handles a full day-plus countdown without wrapping', () => {
		// 25 hours must read 25:00:00, not 01:00:00
		expect(formatCountdown(25 * 3600)).toBe('25:00:00');
	});

	it('clamps negatives to zero', () => {
		expect(formatCountdown(-10)).toBe('00:00:00');
	});
});

describe('formatUntil', () => {
	it('returns "now" at or below zero', () => {
		expect(formatUntil(0)).toBe('now');
		expect(formatUntil(-3)).toBe('now');
	});

	it('uses minutes under an hour, rounding up', () => {
		expect(formatUntil(30)).toBe('in 1m');
		expect(formatUntil(600)).toBe('in 10m');
	});

	it('switches to hours and minutes past an hour', () => {
		expect(formatUntil(3600)).toBe('in 1h 0m');
		expect(formatUntil(3600 * 4 + 12 * 60)).toBe('in 4h 12m');
	});
});
