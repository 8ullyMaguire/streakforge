import { describe, it, expect } from 'vitest';
import { timeAgo, formatCount } from './api';

describe('timeAgo', () => {
	const now = new Date('2026-08-10T12:00:00Z');
	it('returns just now for < 5s', () => {
		expect(timeAgo('2026-08-10T11:59:58Z', now)).toBe('just now');
	});
	it('seconds', () => {
		expect(timeAgo('2026-08-10T11:59:30Z', now)).toBe('30s ago');
	});
	it('minutes', () => {
		expect(timeAgo('2026-08-10T11:45:00Z', now)).toBe('15m ago');
	});
	it('hours', () => {
		expect(timeAgo('2026-08-10T09:00:00Z', now)).toBe('3h ago');
	});
	it('days', () => {
		expect(timeAgo('2026-08-07T12:00:00Z', now)).toBe('3d ago');
	});
	it('older than a week uses locale date', () => {
		expect(timeAgo('2026-07-01T12:00:00Z', now)).toBe(new Date('2026-07-01T12:00:00Z').toLocaleDateString());
	});
});

describe('formatCount', () => {
	it('pads to 8 digits', () => {
		expect(formatCount(62)).toBe('00000062');
	});
	it('handles large numbers', () => {
		expect(formatCount(12345678)).toBe('12345678');
	});
});
