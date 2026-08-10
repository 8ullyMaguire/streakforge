import { describe, it, expect } from 'vitest';
import { isValidUsername } from './validation';

describe('isValidUsername', () => {
	it('accepts alphanumeric + underscore', () => {
		expect(isValidUsername('dev_forger')).toBe(true);
		expect(isValidUsername('ForgeMaster99')).toBe(true);
	});
	it('rejects too short', () => {
		expect(isValidUsername('a')).toBe(false);
	});
	it('rejects too long', () => {
		expect(isValidUsername('a'.repeat(31))).toBe(false);
	});
	it('rejects invalid chars', () => {
		expect(isValidUsername('bad-name')).toBe(false);
		expect(isValidUsername('has space')).toBe(false);
		expect(isValidUsername('emoji🔥')).toBe(false);
	});
});
