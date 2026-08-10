import type { Handle } from '@sveltejs/kit';
import { sequence } from '@sveltejs/kit/hooks';

export const handle: Handle = sequence(async ({ event, resolve }) => {
	// Keep responses lean; nothing special needed for this SPA.
	return resolve(event);
});
