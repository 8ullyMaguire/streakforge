
// this file is generated — do not edit it


declare module "svelte/elements" {
	export interface HTMLAttributes<T> {
		'data-sveltekit-keepfocus'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-noscroll'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-preload-code'?:
			| true
			| ''
			| 'eager'
			| 'viewport'
			| 'hover'
			| 'tap'
			| 'off'
			| undefined
			| null;
		'data-sveltekit-preload-data'?: true | '' | 'hover' | 'tap' | 'off' | undefined | null;
		'data-sveltekit-reload'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-replacestate'?: true | '' | 'off' | undefined | null;
	}
}

export {};


declare module "$app/types" {
	type MatcherParam<M> = M extends (param : string) => param is (infer U extends string) ? U : string;

	export interface AppTypes {
		RouteId(): "/" | "/app" | "/dashboard" | "/denial" | "/doctrine" | "/drill" | "/feed" | "/leaderboard" | "/login" | "/profile" | "/profile/[username]" | "/settings";
		RouteParams(): {
			"/profile/[username]": { username: string }
		};
		LayoutParams(): {
			"/": { username?: string | undefined };
			"/app": Record<string, never>;
			"/dashboard": Record<string, never>;
			"/denial": Record<string, never>;
			"/doctrine": Record<string, never>;
			"/drill": Record<string, never>;
			"/feed": Record<string, never>;
			"/leaderboard": Record<string, never>;
			"/login": Record<string, never>;
			"/profile": { username?: string | undefined };
			"/profile/[username]": { username: string };
			"/settings": Record<string, never>
		};
		Pathname(): "/" | "/app" | "/dashboard" | "/denial" | "/doctrine" | "/drill" | "/feed" | "/leaderboard" | "/login" | `/profile/${string}` & {} | "/settings";
		ResolvedPathname(): `${"" | `/${string}`}${ReturnType<AppTypes['Pathname']>}`;
		Asset(): "/icon.svg" | "/manifest.webmanifest" | string & {};
	}
}