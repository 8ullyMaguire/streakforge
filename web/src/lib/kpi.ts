// Pure helpers for the KPI dashboard page (no DOM, unit-testable).

export interface TrendPoint {
	day: string;
	wasted: number;
	denied: number;
	affirmations: number;
}

/** Of all loads committed, what % were denied. 0 when no loads. */
export function denialRate(wasted: number, denied: number): number {
	const total = wasted + denied;
	if (total === 0) return 0;
	return (denied / total) * 100;
}

/** Largest single-kind count across the trend — bar chart scale. */
export function maxTrend(trend: TrendPoint[]): number {
	return Math.max(1, ...trend.map((t) => Math.max(t.wasted, t.denied, t.affirmations)));
}

/** 1,234,567 formatting for big counters. */
export function compact(n: number): string {
	return n.toLocaleString('en-US');
}
