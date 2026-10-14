// Item 10's dashboard line. Every sentence the user reads is asserted here, on
// both sides of every branch: the empty state, one, and many.
import { describe, it, expect } from 'vitest';
import { todayLine, todayLinkLabel } from './today';

describe('todayLine', () => {
	it('names denials first, in the site\'s own word', () => {
		expect(todayLine({ denials_today: 17, wastes_today: 4, loggers_today: 9 })).toBe(
			'17 whitebois denied today · 4 wasted'
		);
	});

	it('pluralises on one', () => {
		expect(todayLine({ denials_today: 1, wastes_today: 1, loggers_today: 2 })).toBe(
			'1 whiteboi denied today · 1 wasted'
		);
	});

	it('says the day is open when nothing has happened, not "0 denied"', () => {
		// "0 whitebois denied today" reads as an accusation of absence. The empty
		// state is the common one for a new site, so it is written as an invitation.
		expect(todayLine({ denials_today: 0, wastes_today: 0, loggers_today: 0 })).toBe(
			'Nobody has logged today yet. The day is still open.'
		);
	});

	it('still reports when people are logging but not denying', () => {
		expect(todayLine({ denials_today: 0, wastes_today: 3, loggers_today: 2 })).toBe(
			'0 whitebois denied today · 3 wasted'
		);
		expect(todayLine({ denials_today: 0, wastes_today: 0, loggers_today: 1 })).toBe(
			'1 whiteboi logging today · nothing denied yet'
		);
	});
});

describe('todayLinkLabel', () => {
	it('points at the KPI page either way', () => {
		expect(todayLinkLabel({ denials_today: 2, wastes_today: 0, loggers_today: 1 })).toBe(
			'TODAY ON THE KPI PAGE'
		);
		expect(todayLinkLabel({ denials_today: 0, wastes_today: 0, loggers_today: 0 })).toBe(
			'THE KPI PAGE'
		);
	});
});
