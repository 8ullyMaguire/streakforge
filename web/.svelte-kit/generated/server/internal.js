
import root from '../root.js';
import { set_building, set_prerendering } from '$app/env/internal';
import { set_assets } from '$app/paths/internal/server';
import { set_manifest, set_read_implementation } from '__sveltekit/server';
import { set_private_env, set_public_env } from '../../../node_modules/@sveltejs/kit/src/runtime/shared-server.js';
import error from '../shared/error-template.js';

export const options = {
	app_template_contains_nonce: false,
	async: false,
	csp: {"mode":"auto","directives":{"upgrade-insecure-requests":false,"block-all-mixed-content":false},"reportOnly":{"upgrade-insecure-requests":false,"block-all-mixed-content":false}},
	csrf_check_origin: true,
	csrf_trusted_origins: [],
	embedded: false,
	env_public_prefix: 'PUBLIC_',
	env_private_prefix: '',
	hash_routing: false,
	hooks: null, // added lazily, via `get_hooks`
	preload_strategy: "modulepreload",
	root,
	service_worker: false,
	service_worker_options: undefined,
	server_error_boundaries: false,
	templates: {
		app: ({ head, body, assets, nonce, env }) => "<!doctype html>\n<html lang=\"en\">\n\t<head>\n\t\t<meta charset=\"utf-8\" />\n\t\t<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n\t\t<meta name=\"description\" content=\"StreakForge — whiteboi denial tracker. Lock, deny, drill, waste when permitted. The board remembers who knows their place.\" />\n\t\t<meta name=\"theme-color\" content=\"#0a0a0a\" />\n\t\t<meta name=\"apple-mobile-web-app-capable\" content=\"yes\" />\n\t\t<meta name=\"apple-mobile-web-app-status-bar-style\" content=\"black-translucent\" />\n\t\t<meta name=\"apple-mobile-web-app-title\" content=\"StreakForge\" />\n\t\t<title>StreakForge — Embrace Defeat. The Future Is Black.</title>\n\t\t" + head + "\n\t\t<link rel=\"icon\" href=\"data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><text y='.9em' font-size='90'>♠️</text></svg>\" />\n\t\t<link rel=\"manifest\" href=\"/manifest.webmanifest\" />\n\t\t<link rel=\"apple-touch-icon\" href=\"/icon.svg\" />\n\t\t<meta name=\"mobile-web-app-capable\" content=\"yes\" />\n\t\t<link rel=\"preconnect\" href=\"https://fonts.googleapis.com\" />\n\t\t<link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin />\n\t\t<link\n\t\t\thref=\"https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&family=Roboto+Mono:wght@500;600;700&display=swap\"\n\t\t\trel=\"stylesheet\"\n\t\t/>\n\t</head>\n\t<body data-sveltekit-preload-data=\"hover\">\n\t\t<div style=\"display: contents\">" + body + "</div>\n\t</body>\n</html>\n",
		error
	},
	version_hash: "vac57h"
};

export async function get_hooks() {
	let handle;
	let handleFetch;
	let handleError;
	let handleValidationError;
	let init;
	({ handle, handleFetch, handleError, handleValidationError, init } = await import("../../../src/hooks.server.ts"));

	let reroute;
	let transport;
	

	return {
		handle,
		handleFetch,
		handleError,
		handleValidationError,
		init,
		reroute,
		transport
	};
}

export { set_assets, set_building, set_manifest, set_prerendering, set_private_env, set_public_env, set_read_implementation };
