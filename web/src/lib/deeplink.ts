// Deep-link logging (docs/100_IDEAS.md #14): /log?kind=…&note=…
//
// The parsing lives here rather than in the page so the rules can be tested
// without a DOM, and because two of them are safety rules rather than
// formatting preferences:
//
//  * an UNKNOWN kind falls back to a waste AND reports the fallback, so the UI
//    can say "that link was not understood" instead of silently recording a
//    different kind than the link asked for;
//  * a note is truncated to the server's limit (140) rather than sent to be
//    rejected, since a deep link's note is almost always assembled by a script
//    that has no idea what the limit is;
//  * a `next=` target is only honoured when it is a same-site absolute path,
//    so a crafted /login?next=https://elsewhere.example cannot turn the sign-in
//    page into an open redirect.

export const LOG_KINDS = ['habit', 'denial', 'affirmation'] as const;
export type LogKindName = (typeof LOG_KINDS)[number];

/** The server's own note limit (backend/src/api.rs rejects past 140). */
export const NOTE_MAX = 140;

export interface ParsedKind {
	kind: LogKindName;
	/** The link named a kind this site does not have. */
	fellBack: boolean;
}

export function parseKind(raw: string | null | undefined): ParsedKind {
	const v = (raw ?? '').trim().toLowerCase();
	if ((LOG_KINDS as readonly string[]).includes(v)) {
		return { kind: v as LogKindName, fellBack: false };
	}
	return { kind: 'habit', fellBack: v.length > 0 };
}

/** What a log of this kind is called, for the confirmation button. */
export function kindLabel(kind: LogKindName): string {
	if (kind === 'denial') return 'a denial';
	if (kind === 'affirmation') return 'a drill rep';
	return 'a waste';
}

export function clampNote(raw: string | null | undefined, max = NOTE_MAX): string {
	const v = (raw ?? '').replace(/\s+/g, ' ').trim();
	// Counted in code points, not UTF-16 units: the server counts chars(), and
	// an emoji would otherwise be truncated mid-surrogate-pair.
	const chars = Array.from(v);
	return chars.length <= max ? v : chars.slice(0, max).join('');
}

/** A `next=` value is honoured only as a same-site absolute path. */
export function safeNext(raw: string | null | undefined): string {
	const v = (raw ?? '').trim();
	if (!v.startsWith('/') || v.startsWith('//')) return '/dashboard';
	return v;
}
