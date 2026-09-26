// Typo-tolerant affirmation matcher.
//
// The drill requires the user to TYPE the affirmation. We compare their input
// against the target after normalizing: lowercase, strip punctuation and
// whitespace. Then we allow up to MAX_EDITS character edits (Levenshtein).
// This makes "Bnwo is not just a kink its reality present and future"
// (missing punctuation, wrong case) still count, while requiring real typing.

/** Maximum Levenshtein edits allowed on the normalized strings. */
export const MAX_EDITS = 3;

/**
 * Normalize for comparison: lowercase, strip everything that isn't a letter
 * or digit, collapse spaces.
 */
export function normalizeAffirmation(s: string): string {
	return s
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, ' ')
		.trim()
		.replace(/\s+/g, ' ');
}

/**
 * Levenshtein distance between two strings (classic DP, O(n*m)).
 */
export function levenshtein(a: string, b: string): number {
	if (a === b) return 0;
	const m = a.length;
	const n = b.length;
	if (m === 0) return n;
	if (n === 0) return m;
	// Use two rows to keep memory flat.
	let prev = new Array<number>(n + 1);
	let curr = new Array<number>(n + 1);
	for (let j = 0; j <= n; j++) prev[j] = j;
	for (let i = 1; i <= m; i++) {
		curr[0] = i;
		for (let j = 1; j <= n; j++) {
			const cost = a[i - 1] === b[j - 1] ? 0 : 1;
			curr[j] = Math.min(prev[j] + 1, curr[j - 1] + 1, prev[j - 1] + cost);
		}
		[prev, curr] = [curr, prev];
	}
	return prev[n];
}

/**
 * Whether the typed input counts as the target affirmation.
 * Empty input never matches.
 */
export function matchesAffirmation(input: string, target: string): boolean {
	const ni = normalizeAffirmation(input);
	const nt = normalizeAffirmation(target);
	if (ni.length === 0 || nt.length === 0) return false;
	return levenshtein(ni, nt) <= MAX_EDITS;
}

/**
 * Similarity 0..1 for a live progress bar (1 = exact match after normalize).
 */
export function affirmationSimilarity(input: string, target: string): number {
	const ni = normalizeAffirmation(input);
	const nt = normalizeAffirmation(target);
	if (nt.length === 0) return 0;
	if (ni.length === 0) return 0;
	const dist = levenshtein(ni, nt);
	const maxLen = Math.max(ni.length, nt.length);
	return Math.max(0, 1 - dist / maxLen);
}
