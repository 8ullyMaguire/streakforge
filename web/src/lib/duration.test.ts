import { describe, expect, it } from 'vitest';
import {
	formatDuration,
	formatDurationShort,
	parseDurationInput,
	TTC_CHIPS
} from './duration';

describe('formatDuration', () => {
	it('reads like a stopwatch under an hour', () => {
		expect(formatDuration(45)).toBe('45s');
		expect(formatDuration(900)).toBe('15m 00s');
		expect(formatDuration(905)).toBe('15m 05s');
	});

	it('drops the seconds past an hour, because nobody compares those', () => {
		expect(formatDuration(3600)).toBe('1h 00m');
		expect(formatDuration(5400)).toBe('1h 30m');
	});

	it('never renders a negative duration', () => {
		expect(formatDuration(-10)).toBe('0s');
	});
});

describe('formatDurationShort', () => {
	it('fits a table cell', () => {
		expect(formatDurationShort(45)).toBe('45s');
		expect(formatDurationShort(900)).toBe('15m');
		expect(formatDurationShort(5400)).toBe('1.5h');
		expect(formatDurationShort(90000)).toBe('1d');
	});
});

describe('parseDurationInput', () => {
	it('accepts mm:ss, units, and bare minutes', () => {
		expect(parseDurationInput('12:30')).toBe(750);
		expect(parseDurationInput('20m')).toBe(1200);
		expect(parseDurationInput('1h')).toBe(3600);
		expect(parseDurationInput('90s')).toBe(90);
		expect(parseDurationInput('15')).toBe(900);
	});

	it('rejects what is not a duration, instead of guessing', () => {
		expect(parseDurationInput('')).toBeNull();
		expect(parseDurationInput('soon')).toBeNull();
		expect(parseDurationInput('-5m')).toBeNull();
		expect(parseDurationInput('200h')).toBeNull(); // past the one-day cap
	});
});

describe('TTC_CHIPS', () => {
	it('offers common answers in ascending order', () => {
		const secs = TTC_CHIPS.map((c) => c.secs);
		expect(secs).toEqual([...secs].sort((a, b) => a - b));
		expect(secs.every((s) => s > 0 && s <= 86400)).toBe(true);
	});
});
