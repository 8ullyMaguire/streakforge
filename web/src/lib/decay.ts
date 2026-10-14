// The decay sentence on the dashboard.
//
// WHY THIS IS A MODULE AND NOT THREE BRANCHES IN THE TEMPLATE: the notice is
// the only place the 30-day decay rule is explained to a user, and it used to be
// markup with its copy inline -- which meant the plan's claim that it was
// "covered by a component test" was not true of anything a test could reach.
// Returning the exact string the user reads means the test asserts the sentence,
// not a fragment of it.
//
// Three states, and the quiet one matters as much as the loud ones: a notice that
// shows up when nothing is wrong teaches people to ignore it.

export type DecayKind = 'wiped' | 'warning' | 'quiet';

export interface DecayNotice {
	kind: DecayKind;
	/** Exactly the sentence rendered on the dashboard. */
	text: string;
}

export interface DecayInput {
	decayed: boolean;
	days_until_decay: number | null;
	lifetime_total: number;
}

/** The lifetime count, pluralised. */
function logs(n: number): string {
	return `${n} lifetime log${n === 1 ? '' : 's'}`;
}

/**
 * What to say about the score's decay, given the server's devotion block.
 *
 * `days_until_decay` is null when there are no logs to decay from, 0 when the
 * score is already gone, and the countdown otherwise. The warning fires at ten
 * days: late enough to be about the real state, early enough to log once.
 */
export function decayNotice(d: DecayInput): DecayNotice {
	if (d.decayed) {
		return {
			kind: 'wiped',
			text: `Score wiped — 30 days without a log. Your record of ${logs(d.lifetime_total)} is intact, but the score resets with the next log you make.`
		};
	}
	const days = d.days_until_decay;
	if (days !== null && days <= 10) {
		const when =
			days === 0
				? 'Your score resets today — it has been 30 days.'
				: `Your score resets in ${days} day${days === 1 ? '' : 's'}.`;
		return { kind: 'warning', text: `${when} One log keeps the era alive.` };
	}
	return {
		kind: 'quiet',
		text: `${logs(d.lifetime_total)} · score resets if you go 30 days without logging`
	};
}

/** The inline style for a notice: colour is the only difference between them. */
export function decayStyle(kind: DecayKind): string {
	switch (kind) {
		case 'wiped':
			return 'margin-top:10px;padding:9px 11px;border-radius:8px;background:rgba(227,28,35,0.10);border:1px solid rgba(227,28,35,0.35);font-size:12px;line-height:1.5;color:var(--red-soft);';
		case 'warning':
			return 'margin-top:10px;padding:9px 11px;border-radius:8px;background:rgba(212,175,55,0.10);border:1px solid rgba(212,175,55,0.35);font-size:12px;line-height:1.5;color:var(--gold);';
		case 'quiet':
			return 'margin-top:8px;font-size:12px;color:var(--text-dim);';
	}
}
