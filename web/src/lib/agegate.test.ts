// The 18+ acknowledgement's rules (item 08).
import { describe, it, expect } from 'vitest';
import { AGE_KEY, acceptedValue, cookieString, shouldGate } from './agegate';

describe('shouldGate', () => {
	it('gates a visitor who has not acknowledged', () => {
		expect(shouldGate(null)).toBe(true);
		expect(shouldGate(undefined)).toBe(true);
		expect(shouldGate('')).toBe(true);
	});

	it('does not gate a visitor who has', () => {
		expect(shouldGate(acceptedValue())).toBe(false);
	});

	it('treats an unrecognised stored value as NOT acknowledged', () => {
		// Anything but the exact marker means "we do not know", and not knowing
		// is not consent.
		expect(shouldGate('0')).toBe(true);
		expect(shouldGate('yes')).toBe(true);
		expect(shouldGate('{"ok":true}')).toBe(true);
	});
});

describe('cookieString', () => {
	it('is a same-site path cookie with a long life', () => {
		const c = cookieString();
		expect(c).toContain(`${AGE_KEY}=${acceptedValue()}`);
		expect(c).toContain('Path=/');
		expect(c).toContain('SameSite=Lax');
		expect(c).toContain('Max-Age=31536000');
	});
});
