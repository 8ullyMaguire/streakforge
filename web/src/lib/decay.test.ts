import { describe, expect, it } from 'vitest';
import { decayNotice, decayStyle } from './decay';

// The four states a user can be in. The point of these tests is the SENTENCE,
// not a fragment: what the dashboard prints is what is asserted here, so a copy
// change has to come through this file. (Item 16's plan line claimed this was
// "covered by a component test" while the copy lived inline in the template and
// nothing could reach it.)
describe('decay notice', () => {
	it('says nothing alarming while the score is healthy', () => {
		const n = decayNotice({ decayed: false, days_until_decay: 30, lifetime_total: 2 });
		expect(n.kind).toBe('quiet');
		expect(n.text).toBe('2 lifetime logs · score resets if you go 30 days without logging');
	});

	it('stays quiet at 11 days, where a warning would be noise', () => {
		// The threshold is the whole feature: warning too early turns the notice
		// into furniture.
		expect(decayNotice({ decayed: false, days_until_decay: 11, lifetime_total: 9 }).kind).toBe('quiet');
	});

	it('warns on the tenth day, counted, with the singular handled', () => {
		expect(decayNotice({ decayed: false, days_until_decay: 10, lifetime_total: 4 }).text).toBe(
			'Your score resets in 10 days. One log keeps the era alive.'
		);
		expect(decayNotice({ decayed: false, days_until_decay: 1, lifetime_total: 4 }).text).toBe(
			'Your score resets in 1 day. One log keeps the era alive.'
		);
	});

	it('names today when the countdown reaches zero', () => {
		expect(decayNotice({ decayed: false, days_until_decay: 0, lifetime_total: 4 }).text).toBe(
			'Your score resets today — it has been 30 days. One log keeps the era alive.'
		);
	});

	it('explains a score that is already gone instead of leaving a zero', () => {
		const n = decayNotice({ decayed: true, days_until_decay: 0, lifetime_total: 1 });
		expect(n.kind).toBe('wiped');
		expect(n.text).toBe(
			'Score wiped — 30 days without a log. Your record of 1 lifetime log is intact, but the score resets with the next log you make.'
		);
	});

	it('has no countdown to give when there are no logs at all', () => {
		// days_until_decay is null for a brand-new account: nothing to decay from,
		// so nothing to warn about.
		const n = decayNotice({ decayed: false, days_until_decay: null, lifetime_total: 0 });
		expect(n.kind).toBe('quiet');
		expect(n.text).toBe('0 lifetime logs · score resets if you go 30 days without logging');
	});

	it('colours the two loud states differently and the quiet one not at all', () => {
		expect(decayStyle('wiped')).toContain('227,28,35');
		expect(decayStyle('warning')).toContain('212,175,55');
		expect(decayStyle('quiet')).not.toContain('border:1px');
	});
});
