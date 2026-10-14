// The community line: who did what today (UTC).
//
// WHY IT IS A MODULE: the first version of this line lived in the template of one
// page, which is the same trap the decay notice fell into -- copy inside markup is
// copy no test can reach, and the plan then says "covered by a component test"
// about a sentence nothing asserts. The sentences are here, and today.test.ts
// asserts them.
//
// WHAT IT IS FOR (docs/100_IDEAS.md #10): logging is solitary, and the number
// kpi_totals already counts is the cheapest way to say that other people are
// doing it too. Denials lead, because a denial is the thing this site is for.

export interface TodayCounts {
	denials_today: number;
	wastes_today: number;
	loggers_today: number;
}

/** `1 whiteboi` / `4 whitebois`. The site's own word, so it is used here too. */
function whitebois(n: number): string {
	return `${n} whiteboi${n === 1 ? '' : 's'}`;
}

/**
 * The line for the dashboard. Empty is a real state and says something useful
 * rather than "0 denied today", which reads as an accusation of absence.
 */
export function todayLine(t: TodayCounts): string {
	const { denials_today: d, wastes_today: w, loggers_today: l } = t;
	if (d === 0 && w === 0) {
		return l > 0
			? `${whitebois(l)} logging today · nothing denied yet`
			: 'Nobody has logged today yet. The day is still open.';
	}
	return `${whitebois(d)} denied today · ${w} wasted`;
}

/** The same numbers as a link label, for the KPI page. */
export function todayLinkLabel(t: TodayCounts): string {
	return t.denials_today + t.wastes_today > 0 ? 'TODAY ON THE KPI PAGE' : 'THE KPI PAGE';
}
