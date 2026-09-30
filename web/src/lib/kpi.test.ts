import { describe, expect, it } from 'vitest';
import { denialRate, maxTrend, compact } from './kpi';

describe('kpi helpers', () => {
	it('denialRate: denied / (wasted+denied) * 100', () => {
		expect(denialRate(3, 1)).toBeCloseTo(25);
		expect(denialRate(0, 0)).toBe(0);
		expect(denialRate(0, 1)).toBe(100);
	});
	it('maxTrend: largest per-kind total for bar scaling', () => {
		const trend = [
			{ day: 'a', wasted: 2, denied: 1, affirmations: 3 },
			{ day: 'b', wasted: 5, denied: 0, affirmations: 0 }
		];
		expect(maxTrend(trend)).toBe(5);
	});
	it('compact: thousands separators', () => {
		expect(compact(0)).toBe('0');
		expect(compact(1234)).toBe('1,234');
		expect(compact(1234567)).toBe('1,234,567');
	});
});
