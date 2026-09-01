
// this file is generated — do not edit it


/// <reference types="@sveltejs/kit" />

/**
 * This module provides access to environment variables that are injected _statically_ into your bundle at build time and are limited to _private_ access.
 * 
 * |         | Runtime                                                                    | Build time                                                               |
 * | ------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
 * | Private | [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private) | [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private) |
 * | Public  | [`$env/dynamic/public`](https://svelte.dev/docs/kit/$env-dynamic-public)   | [`$env/static/public`](https://svelte.dev/docs/kit/$env-static-public)   |
 * 
 * Static environment variables are [loaded by Vite](https://vitejs.dev/guide/env-and-mode.html#env-files) from `.env` files and `process.env` at build time and then statically injected into your bundle at build time, enabling optimisations like dead code elimination.
 * 
 * **_Private_ access:**
 * 
 * - This module cannot be imported into client-side code
 * - This module only includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured)
 * 
 * For example, given the following build time environment:
 * 
 * ```env
 * ENVIRONMENT=production
 * PUBLIC_BASE_URL=http://site.com
 * ```
 * 
 * With the default `publicPrefix` and `privatePrefix`:
 * 
 * ```ts
 * import { ENVIRONMENT, PUBLIC_BASE_URL } from '$env/static/private';
 * 
 * console.log(ENVIRONMENT); // => "production"
 * console.log(PUBLIC_BASE_URL); // => throws error during build
 * ```
 * 
 * The above values will be the same _even if_ different values for `ENVIRONMENT` or `PUBLIC_BASE_URL` are set at runtime, as they are statically replaced in your code with their build time values.
 */
declare module '$env/static/private' {
	export const INIT_CWD: string;
	export const NPM_CONFIG_PREFIX: string;
	export const HERMES_CUSTOM_API_COMMANDCODE_AI_API_KEY: string;
	export const EDITOR: string;
	export const VAGRANT_HOME: string;
	export const TERMINAL_HOME_MODE: string;
	export const VIM_TMP: string;
	export const GEM_SPEC_CACHE: string;
	export const QT_IM_MODULES: string;
	export const BROWSER_INACTIVITY_TIMEOUT: string;
	export const IPYTHONDIR: string;
	export const PROMPT_EOL_MARK: string;
	export const MOA_TOOLS_DEBUG: string;
	export const LOCALENV: string;
	export const ZSH_CACHE_DIR: string;
	export const ATOM_HOME: string;
	export const XDG_CONFIG_DIRS: string;
	export const TERMINFO_DIRS: string;
	export const TMUX: string;
	export const CONDARC: string;
	export const npm_config_npm_version: string;
	export const XAUTHORITY: string;
	export const HERMES_QUIET: string;
	export const IMAGE_TOOLS_DEBUG: string;
	export const ZSH_COMPDUMP: string;
	export const HERMES_COMPRESS_AFTER_TURNS: string;
	export const TERMINAL_DOCKER_VOLUMES: string;
	export const LOGNAME: string;
	export const SHELL: string;
	export const DOCKER_CONFIG: string;
	export const TERMINAL_VERCEL_RUNTIME: string;
	export const _HERMES_GATEWAY: string;
	export const NVM_DIR: string;
	export const PIEFED_PASSWORD: string;
	export const HERMES_TURN_LEASE_TIMEOUT: string;
	export const XDG_RUNTIME_DIR: string;
	export const WGETRC: string;
	export const READER: string;
	export const HERMES_PLAYBOOK_DIR: string;
	export const HERMES_MEMORY_SEARCH_TOP_K: string;
	export const HERMES_CODE_MODEL: string;
	export const SCREENRC: string;
	export const NUGET_PACKAGES: string;
	export const HERMES_INTERACTIVE: string;
	export const GIO_LAUNCHED_DESKTOP_FILE_PID: string;
	export const ZSHRC: string;
	export const TERMINAL_MODAL_IMAGE: string;
	export const ZEPH_HOME: string;
	export const VISUAL: string;
	export const TERMINFO: string;
	export const WINDOWID: string;
	export const HERMES_EMBED_MODEL: string;
	export const PATH: string;
	export const npm_config_local_prefix: string;
	export const HERMES_ALWAYS_LOAD: string;
	export const TERMINAL_CONTAINER_PERSISTENT: string;
	export const TERMINAL_ENV: string;
	export const HERMES_MAX_ITERATIONS: string;
	export const npm_execpath: string;
	export const LC_MONETARY: string;
	export const TERM: string;
	export const npm_config_cache: string;
	export const GITRC: string;
	export const HERMES_REAL_HOME: string;
	export const BROWSERBASE_ADVANCED_STEALTH: string;
	export const XDG_GREETER_DATA_DIR: string;
	export const KEYTIMEOUT: string;
	export const LC_NUMERIC: string;
	export const TERMINAL_DOCKER_MOUNT_CWD_TO_WORKSPACE: string;
	export const XDG_SEAT_PATH: string;
	export const CLUTTER_IM_MODULE: string;
	export const _ZSH_TMUX_FIXED_CONFIG: string;
	export const GTK_MODULES: string;
	export const LOCALRC: string;
	export const XINITRC: string;
	export const npm_config_node_gyp: string;
	export const XDG_SEAT: string;
	export const TERMINAL_DAYTONA_IMAGE: string;
	export const COLOR: string;
	export const HERMES_NOTES_DIR: string;
	export const MYSQL_HISTFILE: string;
	export const ZSH_TMUX_CONFIG: string;
	export const BUNDLE_USER_CACHE: string;
	export const HERMES_AGENT: string;
	export const HERMES_MEMORY_DB: string;
	export const HISTFILE: string;
	export const XDG_CACHE_HOME: string;
	export const RUSTUP_HOME: string;
	export const TERMINAL_DOCKER_ENV: string;
	export const HERMES_TOKEN_OPTIMIZER_DEFAULT: string;
	export const _: string;
	export const MUTT: string;
	export const VIM_DIR: string;
	export const GIO_LAUNCHED_DESKTOP_FILE: string;
	export const npm_config_user_agent: string;
	export const GJS_DEBUG_TOPICS: string;
	export const XDG_STATE_HOME: string;
	export const SESSION_MANAGER: string;
	export const NSS_USER_PKI: string;
	export const LC_PAPER: string;
	export const TERMINAL_DOCKER_IMAGE: string;
	export const RIPGREP_CONFIG_PATH: string;
	export const JUPYTER_CONFIG_DIR: string;
	export const HERMES_TOKEN_OPTIMIZER: string;
	export const HERMES_MAX_TOOL_OUTPUT_CHARS: string;
	export const TERMINAL_CONTAINER_MEMORY: string;
	export const DESKTOP_SESSION: string;
	export const npm_package_name: string;
	export const QT_IM_MODULE: string;
	export const QT_QPA_PLATFORMTHEME: string;
	export const WINEPREFIX: string;
	export const npm_config_tmp: string;
	export const BSKY_EN_APP_PASSWORD: string;
	export const WEB_TOOLS_DEBUG: string;
	export const DOWNLOADS: string;
	export const HERMES_PROMPT_CACHE: string;
	export const SYNC: string;
	export const COMMANDCODE_API_KEY: string;
	export const TERMINAL_DOCKER_FORWARD_ENV: string;
	export const ALACRITTY_LOG: string;
	export const PGPASSFILE: string;
	export const LEMMY_USERNAME: string;
	export const HERMES_HOME: string;
	export const GIT_EDITOR: string;
	export const XDG_DATA_HOME: string;
	export const BROWSERBASE_PROXIES: string;
	export const FORGEJO_TOKEN: string;
	export const DBUS_SESSION_BUS_ADDRESS: string;
	export const COLORTERM: string;
	export const USER: string;
	export const NPM_BIN: string;
	export const OLLAMA_ENDPOINT: string;
	export const BUNDLE_USER_CONFIG: string;
	export const PASSWORD_STORE_DIR: string;
	export const PSQL_HISTORY: string;
	export const TERMINAL_CWD: string;
	export const XMODIFIERS: string;
	export const BUNDLE_USER_PLUGIN: string;
	export const XDG_SESSION_TYPE: string;
	export const ZDOTDIR: string;
	export const npm_config_noproxy: string;
	export const LESSKEY: string;
	export const LESS: string;
	export const NODE: string;
	export const OLDPWD: string;
	export const VIPSHOME: string;
	export const NPM_PATH: string;
	export const TMUX_DIR: string;
	export const BROWSER_SESSION_TIMEOUT: string;
	export const ZSHENV: string;
	export const AZURACAST_URL: string;
	export const LC_NAME: string;
	export const npm_command: string;
	export const LANGUAGE: string;
	export const HERMES_KANBAN_BOARD: string;
	export const GNOME_DESKTOP_SESSION_ID: string;
	export const GTK_IM_MODULE: string;
	export const ALACRITTY_SOCKET: string;
	export const HERMES_CONFIG_HOME: string;
	export const LSCOLORS: string;
	export const HERMES_CONTEXT_BUDGET_PERCENT: string;
	export const npm_package_json: string;
	export const XDG_SESSION_ID: string;
	export const CHOOSENIM_DIR: string;
	export const TERMINAL_SINGULARITY_IMAGE: string;
	export const INPUTRC: string;
	export const HERMES_EPISODES_DIR: string;
	export const TERMINAL_TIMEOUT: string;
	export const LANG: string;
	export const NODE_ENV: string;
	export const AI_AGENT: string;
	export const STREAKFORGE_SESSION_SECRET: string;
	export const HERMES_EXEC_ASK: string;
	export const FORGEJO_USERNAME: string;
	export const SSL_CERT_FILE: string;
	export const ZSH: string;
	export const TERMINAL_PERSISTENT_SHELL: string;
	export const ANDROID_USER_HOME: string;
	export const LOCALPRO: string;
	export const TERMINAL_LIFETIME_SECONDS: string;
	export const BSKY_EN_HANDLE: string;
	export const MYPY_CACHE_DIR: string;
	export const GTK2_RC_FILES: string;
	export const HERMES_SMALL_MODEL: string;
	export const BSKY_HANDLE: string;
	export const GPG_AGENT_INFO: string;
	export const LESSHISTFILE: string;
	export const GOBIN: string;
	export const NIMBLE_DIR: string;
	export const GDM_LANG: string;
	export const TERMINAL_DOCKER_NETWORK: string;
	export const NPM_CONFIG_USERCONFIG: string;
	export const GTK3_MODULES: string;
	export const GDMSESSION: string;
	export const LC_ADDRESS: string;
	export const CINNAMON_VERSION: string;
	export const TODOTXT_CFG_FILE: string;
	export const DISPLAY: string;
	export const TERMINAL_DOCKER_SHM_SIZE: string;
	export const XDG_SESSION_CLASS: string;
	export const PIEFED_INSTANCE: string;
	export const HISTSIZE: string;
	export const npm_node_execpath: string;
	export const XDG_CURRENT_DESKTOP: string;
	export const LC_MEASUREMENT: string;
	export const LC_TELEPHONE: string;
	export const LEMMY_PASSWORD: string;
	export const SAVEHIST: string;
	export const XDG_CONFIG_HOME: string;
	export const XDG_SESSION_DESKTOP: string;
	export const VAGRANT_ALIAS_FILE: string;
	export const LS_COLORS: string;
	export const HERMES_HEADROOM_PERCENT: string;
	export const TMUXRC: string;
	export const HOME: string;
	export const TERM_PROGRAM: string;
	export const PAGER: string;
	export const npm_lifecycle_script: string;
	export const DOT: string;
	export const FORGEJO_PASSWORD: string;
	export const LC_IDENTIFICATION: string;
	export const ACKRC: string;
	export const OLLAMA_MODELS: string;
	export const ZSH_CUSTOM: string;
	export const GNUPGHOME: string;
	export const npm_config_global_prefix: string;
	export const HERMES_AUTO_CAPTURE: string;
	export const NODE_REPL_HISTORY: string;
	export const MACHINE_STORAGE_PATH: string;
	export const HERMES_SECOND_BRAIN: string;
	export const HERMES_SESSION_ID: string;
	export const SCREENSHOT: string;
	export const TERMINAL_DOCKER_EXTRA_ARGS: string;
	export const STREAKFORGE_DB_PASSWORD: string;
	export const TERMINAL_DEGRADED_MODE: string;
	export const ZSH_TMUX_TERM: string;
	export const TERMINAL_MODAL_MODE: string;
	export const GOCACHE: string;
	export const XDG_SESSION_PATH: string;
	export const GOPATH: string;
	export const npm_package_version: string;
	export const QT_ACCESSIBILITY: string;
	export const MYVIMRC: string;
	export const VISION_TOOLS_DEBUG: string;
	export const TERMINAL_CONTAINER_CPU: string;
	export const GEM_HOME: string;
	export const WIKI_PATH: string;
	export const _JAVA_OPTIONS: string;
	export const AZURACAST_PASSWORD: string;
	export const CONDA_CHANGEPS1: string;
	export const MUTTRC: string;
	export const LEMMY_URL: string;
	export const HERMES_CONTEXT_DIR: string;
	export const TERMINAL_CONTAINER_DISK: string;
	export const GJS_DEBUG_OUTPUT: string;
	export const AZURACAST_EMAIL: string;
	export const _Z_DATA: string;
	export const CARGO_REGISTRY_TOKEN: string;
	export const ZPROFILE: string;
	export const TERMINAL_DOCKER_RUN_AS_HOST_USER: string;
	export const npm_config_init_module: string;
	export const SVELTEKIT_FORK: string;
	export const XDG_VTNR: string;
	export const SHLVL: string;
	export const ALACRITTY_WINDOW_ID: string;
	export const npm_config_globalconfig: string;
	export const PWD: string;
	export const GRADLE_USER_HOME: string;
	export const TMUX_PANE: string;
	export const npm_lifecycle_event: string;
	export const PSQLRC: string;
	export const TERM_PROGRAM_VERSION: string;
	export const _config_version: string;
	export const PIEFED_USERNAME: string;
	export const CARGO_HOME: string;
	export const CONFIG_DIR: string;
	export const BSKY_APP_PASSWORD: string;
	export const SSH_AUTH_SOCK: string;
	export const VIRTUAL_ENV_DISABLE_PROMPT: string;
	export const XDG_DATA_DIRS: string;
	export const FORGEJO_INSTANCE: string;
}

/**
 * This module provides access to environment variables that are injected _statically_ into your bundle at build time and are _publicly_ accessible.
 * 
 * |         | Runtime                                                                    | Build time                                                               |
 * | ------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
 * | Private | [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private) | [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private) |
 * | Public  | [`$env/dynamic/public`](https://svelte.dev/docs/kit/$env-dynamic-public)   | [`$env/static/public`](https://svelte.dev/docs/kit/$env-static-public)   |
 * 
 * Static environment variables are [loaded by Vite](https://vitejs.dev/guide/env-and-mode.html#env-files) from `.env` files and `process.env` at build time and then statically injected into your bundle at build time, enabling optimisations like dead code elimination.
 * 
 * **_Public_ access:**
 * 
 * - This module _can_ be imported into client-side code
 * - **Only** variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`) are included
 * 
 * For example, given the following build time environment:
 * 
 * ```env
 * ENVIRONMENT=production
 * PUBLIC_BASE_URL=http://site.com
 * ```
 * 
 * With the default `publicPrefix` and `privatePrefix`:
 * 
 * ```ts
 * import { ENVIRONMENT, PUBLIC_BASE_URL } from '$env/static/public';
 * 
 * console.log(ENVIRONMENT); // => throws error during build
 * console.log(PUBLIC_BASE_URL); // => "http://site.com"
 * ```
 * 
 * The above values will be the same _even if_ different values for `ENVIRONMENT` or `PUBLIC_BASE_URL` are set at runtime, as they are statically replaced in your code with their build time values.
 */
declare module '$env/static/public' {
	
}

/**
 * This module provides access to environment variables set _dynamically_ at runtime and that are limited to _private_ access.
 * 
 * |         | Runtime                                                                    | Build time                                                               |
 * | ------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
 * | Private | [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private) | [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private) |
 * | Public  | [`$env/dynamic/public`](https://svelte.dev/docs/kit/$env-dynamic-public)   | [`$env/static/public`](https://svelte.dev/docs/kit/$env-static-public)   |
 * 
 * Dynamic environment variables are defined by the platform you're running on. For example if you're using [`adapter-node`](https://github.com/sveltejs/kit/tree/main/packages/adapter-node) (or running [`vite preview`](https://svelte.dev/docs/kit/cli)), this is equivalent to `process.env`.
 * 
 * **_Private_ access:**
 * 
 * - This module cannot be imported into client-side code
 * - This module includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured)
 * 
 * > [!NOTE] In `dev`, `$env/dynamic` includes environment variables from `.env`. In `prod`, this behavior will depend on your adapter.
 * 
 * > [!NOTE] To get correct types, environment variables referenced in your code should be declared (for example in an `.env` file), even if they don't have a value until the app is deployed:
 * >
 * > ```env
 * > MY_FEATURE_FLAG=
 * > ```
 * >
 * > You can override `.env` values from the command line like so:
 * >
 * > ```sh
 * > MY_FEATURE_FLAG="enabled" npm run dev
 * > ```
 * 
 * For example, given the following runtime environment:
 * 
 * ```env
 * ENVIRONMENT=production
 * PUBLIC_BASE_URL=http://site.com
 * ```
 * 
 * With the default `publicPrefix` and `privatePrefix`:
 * 
 * ```ts
 * import { env } from '$env/dynamic/private';
 * 
 * console.log(env.ENVIRONMENT); // => "production"
 * console.log(env.PUBLIC_BASE_URL); // => undefined
 * ```
 */
declare module '$env/dynamic/private' {
	export const env: {
		INIT_CWD: string;
		NPM_CONFIG_PREFIX: string;
		HERMES_CUSTOM_API_COMMANDCODE_AI_API_KEY: string;
		EDITOR: string;
		VAGRANT_HOME: string;
		TERMINAL_HOME_MODE: string;
		VIM_TMP: string;
		GEM_SPEC_CACHE: string;
		QT_IM_MODULES: string;
		BROWSER_INACTIVITY_TIMEOUT: string;
		IPYTHONDIR: string;
		PROMPT_EOL_MARK: string;
		MOA_TOOLS_DEBUG: string;
		LOCALENV: string;
		ZSH_CACHE_DIR: string;
		ATOM_HOME: string;
		XDG_CONFIG_DIRS: string;
		TERMINFO_DIRS: string;
		TMUX: string;
		CONDARC: string;
		npm_config_npm_version: string;
		XAUTHORITY: string;
		HERMES_QUIET: string;
		IMAGE_TOOLS_DEBUG: string;
		ZSH_COMPDUMP: string;
		HERMES_COMPRESS_AFTER_TURNS: string;
		TERMINAL_DOCKER_VOLUMES: string;
		LOGNAME: string;
		SHELL: string;
		DOCKER_CONFIG: string;
		TERMINAL_VERCEL_RUNTIME: string;
		_HERMES_GATEWAY: string;
		NVM_DIR: string;
		PIEFED_PASSWORD: string;
		HERMES_TURN_LEASE_TIMEOUT: string;
		XDG_RUNTIME_DIR: string;
		WGETRC: string;
		READER: string;
		HERMES_PLAYBOOK_DIR: string;
		HERMES_MEMORY_SEARCH_TOP_K: string;
		HERMES_CODE_MODEL: string;
		SCREENRC: string;
		NUGET_PACKAGES: string;
		HERMES_INTERACTIVE: string;
		GIO_LAUNCHED_DESKTOP_FILE_PID: string;
		ZSHRC: string;
		TERMINAL_MODAL_IMAGE: string;
		ZEPH_HOME: string;
		VISUAL: string;
		TERMINFO: string;
		WINDOWID: string;
		HERMES_EMBED_MODEL: string;
		PATH: string;
		npm_config_local_prefix: string;
		HERMES_ALWAYS_LOAD: string;
		TERMINAL_CONTAINER_PERSISTENT: string;
		TERMINAL_ENV: string;
		HERMES_MAX_ITERATIONS: string;
		npm_execpath: string;
		LC_MONETARY: string;
		TERM: string;
		npm_config_cache: string;
		GITRC: string;
		HERMES_REAL_HOME: string;
		BROWSERBASE_ADVANCED_STEALTH: string;
		XDG_GREETER_DATA_DIR: string;
		KEYTIMEOUT: string;
		LC_NUMERIC: string;
		TERMINAL_DOCKER_MOUNT_CWD_TO_WORKSPACE: string;
		XDG_SEAT_PATH: string;
		CLUTTER_IM_MODULE: string;
		_ZSH_TMUX_FIXED_CONFIG: string;
		GTK_MODULES: string;
		LOCALRC: string;
		XINITRC: string;
		npm_config_node_gyp: string;
		XDG_SEAT: string;
		TERMINAL_DAYTONA_IMAGE: string;
		COLOR: string;
		HERMES_NOTES_DIR: string;
		MYSQL_HISTFILE: string;
		ZSH_TMUX_CONFIG: string;
		BUNDLE_USER_CACHE: string;
		HERMES_AGENT: string;
		HERMES_MEMORY_DB: string;
		HISTFILE: string;
		XDG_CACHE_HOME: string;
		RUSTUP_HOME: string;
		TERMINAL_DOCKER_ENV: string;
		HERMES_TOKEN_OPTIMIZER_DEFAULT: string;
		_: string;
		MUTT: string;
		VIM_DIR: string;
		GIO_LAUNCHED_DESKTOP_FILE: string;
		npm_config_user_agent: string;
		GJS_DEBUG_TOPICS: string;
		XDG_STATE_HOME: string;
		SESSION_MANAGER: string;
		NSS_USER_PKI: string;
		LC_PAPER: string;
		TERMINAL_DOCKER_IMAGE: string;
		RIPGREP_CONFIG_PATH: string;
		JUPYTER_CONFIG_DIR: string;
		HERMES_TOKEN_OPTIMIZER: string;
		HERMES_MAX_TOOL_OUTPUT_CHARS: string;
		TERMINAL_CONTAINER_MEMORY: string;
		DESKTOP_SESSION: string;
		npm_package_name: string;
		QT_IM_MODULE: string;
		QT_QPA_PLATFORMTHEME: string;
		WINEPREFIX: string;
		npm_config_tmp: string;
		BSKY_EN_APP_PASSWORD: string;
		WEB_TOOLS_DEBUG: string;
		DOWNLOADS: string;
		HERMES_PROMPT_CACHE: string;
		SYNC: string;
		COMMANDCODE_API_KEY: string;
		TERMINAL_DOCKER_FORWARD_ENV: string;
		ALACRITTY_LOG: string;
		PGPASSFILE: string;
		LEMMY_USERNAME: string;
		HERMES_HOME: string;
		GIT_EDITOR: string;
		XDG_DATA_HOME: string;
		BROWSERBASE_PROXIES: string;
		FORGEJO_TOKEN: string;
		DBUS_SESSION_BUS_ADDRESS: string;
		COLORTERM: string;
		USER: string;
		NPM_BIN: string;
		OLLAMA_ENDPOINT: string;
		BUNDLE_USER_CONFIG: string;
		PASSWORD_STORE_DIR: string;
		PSQL_HISTORY: string;
		TERMINAL_CWD: string;
		XMODIFIERS: string;
		BUNDLE_USER_PLUGIN: string;
		XDG_SESSION_TYPE: string;
		ZDOTDIR: string;
		npm_config_noproxy: string;
		LESSKEY: string;
		LESS: string;
		NODE: string;
		OLDPWD: string;
		VIPSHOME: string;
		NPM_PATH: string;
		TMUX_DIR: string;
		BROWSER_SESSION_TIMEOUT: string;
		ZSHENV: string;
		AZURACAST_URL: string;
		LC_NAME: string;
		npm_command: string;
		LANGUAGE: string;
		HERMES_KANBAN_BOARD: string;
		GNOME_DESKTOP_SESSION_ID: string;
		GTK_IM_MODULE: string;
		ALACRITTY_SOCKET: string;
		HERMES_CONFIG_HOME: string;
		LSCOLORS: string;
		HERMES_CONTEXT_BUDGET_PERCENT: string;
		npm_package_json: string;
		XDG_SESSION_ID: string;
		CHOOSENIM_DIR: string;
		TERMINAL_SINGULARITY_IMAGE: string;
		INPUTRC: string;
		HERMES_EPISODES_DIR: string;
		TERMINAL_TIMEOUT: string;
		LANG: string;
		NODE_ENV: string;
		AI_AGENT: string;
		STREAKFORGE_SESSION_SECRET: string;
		HERMES_EXEC_ASK: string;
		FORGEJO_USERNAME: string;
		SSL_CERT_FILE: string;
		ZSH: string;
		TERMINAL_PERSISTENT_SHELL: string;
		ANDROID_USER_HOME: string;
		LOCALPRO: string;
		TERMINAL_LIFETIME_SECONDS: string;
		BSKY_EN_HANDLE: string;
		MYPY_CACHE_DIR: string;
		GTK2_RC_FILES: string;
		HERMES_SMALL_MODEL: string;
		BSKY_HANDLE: string;
		GPG_AGENT_INFO: string;
		LESSHISTFILE: string;
		GOBIN: string;
		NIMBLE_DIR: string;
		GDM_LANG: string;
		TERMINAL_DOCKER_NETWORK: string;
		NPM_CONFIG_USERCONFIG: string;
		GTK3_MODULES: string;
		GDMSESSION: string;
		LC_ADDRESS: string;
		CINNAMON_VERSION: string;
		TODOTXT_CFG_FILE: string;
		DISPLAY: string;
		TERMINAL_DOCKER_SHM_SIZE: string;
		XDG_SESSION_CLASS: string;
		PIEFED_INSTANCE: string;
		HISTSIZE: string;
		npm_node_execpath: string;
		XDG_CURRENT_DESKTOP: string;
		LC_MEASUREMENT: string;
		LC_TELEPHONE: string;
		LEMMY_PASSWORD: string;
		SAVEHIST: string;
		XDG_CONFIG_HOME: string;
		XDG_SESSION_DESKTOP: string;
		VAGRANT_ALIAS_FILE: string;
		LS_COLORS: string;
		HERMES_HEADROOM_PERCENT: string;
		TMUXRC: string;
		HOME: string;
		TERM_PROGRAM: string;
		PAGER: string;
		npm_lifecycle_script: string;
		DOT: string;
		FORGEJO_PASSWORD: string;
		LC_IDENTIFICATION: string;
		ACKRC: string;
		OLLAMA_MODELS: string;
		ZSH_CUSTOM: string;
		GNUPGHOME: string;
		npm_config_global_prefix: string;
		HERMES_AUTO_CAPTURE: string;
		NODE_REPL_HISTORY: string;
		MACHINE_STORAGE_PATH: string;
		HERMES_SECOND_BRAIN: string;
		HERMES_SESSION_ID: string;
		SCREENSHOT: string;
		TERMINAL_DOCKER_EXTRA_ARGS: string;
		STREAKFORGE_DB_PASSWORD: string;
		TERMINAL_DEGRADED_MODE: string;
		ZSH_TMUX_TERM: string;
		TERMINAL_MODAL_MODE: string;
		GOCACHE: string;
		XDG_SESSION_PATH: string;
		GOPATH: string;
		npm_package_version: string;
		QT_ACCESSIBILITY: string;
		MYVIMRC: string;
		VISION_TOOLS_DEBUG: string;
		TERMINAL_CONTAINER_CPU: string;
		GEM_HOME: string;
		WIKI_PATH: string;
		_JAVA_OPTIONS: string;
		AZURACAST_PASSWORD: string;
		CONDA_CHANGEPS1: string;
		MUTTRC: string;
		LEMMY_URL: string;
		HERMES_CONTEXT_DIR: string;
		TERMINAL_CONTAINER_DISK: string;
		GJS_DEBUG_OUTPUT: string;
		AZURACAST_EMAIL: string;
		_Z_DATA: string;
		CARGO_REGISTRY_TOKEN: string;
		ZPROFILE: string;
		TERMINAL_DOCKER_RUN_AS_HOST_USER: string;
		npm_config_init_module: string;
		SVELTEKIT_FORK: string;
		XDG_VTNR: string;
		SHLVL: string;
		ALACRITTY_WINDOW_ID: string;
		npm_config_globalconfig: string;
		PWD: string;
		GRADLE_USER_HOME: string;
		TMUX_PANE: string;
		npm_lifecycle_event: string;
		PSQLRC: string;
		TERM_PROGRAM_VERSION: string;
		_config_version: string;
		PIEFED_USERNAME: string;
		CARGO_HOME: string;
		CONFIG_DIR: string;
		BSKY_APP_PASSWORD: string;
		SSH_AUTH_SOCK: string;
		VIRTUAL_ENV_DISABLE_PROMPT: string;
		XDG_DATA_DIRS: string;
		FORGEJO_INSTANCE: string;
		[key: `PUBLIC_${string}`]: undefined;
		[key: `${string}`]: string | undefined;
	}
}

/**
 * This module provides access to environment variables set _dynamically_ at runtime and that are _publicly_ accessible.
 * 
 * |         | Runtime                                                                    | Build time                                                               |
 * | ------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
 * | Private | [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private) | [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private) |
 * | Public  | [`$env/dynamic/public`](https://svelte.dev/docs/kit/$env-dynamic-public)   | [`$env/static/public`](https://svelte.dev/docs/kit/$env-static-public)   |
 * 
 * Dynamic environment variables are defined by the platform you're running on. For example if you're using [`adapter-node`](https://github.com/sveltejs/kit/tree/main/packages/adapter-node) (or running [`vite preview`](https://svelte.dev/docs/kit/cli)), this is equivalent to `process.env`.
 * 
 * **_Public_ access:**
 * 
 * - This module _can_ be imported into client-side code
 * - **Only** variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`) are included
 * 
 * > [!NOTE] In `dev`, `$env/dynamic` includes environment variables from `.env`. In `prod`, this behavior will depend on your adapter.
 * 
 * > [!NOTE] To get correct types, environment variables referenced in your code should be declared (for example in an `.env` file), even if they don't have a value until the app is deployed:
 * >
 * > ```env
 * > MY_FEATURE_FLAG=
 * > ```
 * >
 * > You can override `.env` values from the command line like so:
 * >
 * > ```sh
 * > MY_FEATURE_FLAG="enabled" npm run dev
 * > ```
 * 
 * For example, given the following runtime environment:
 * 
 * ```env
 * ENVIRONMENT=production
 * PUBLIC_BASE_URL=http://example.com
 * ```
 * 
 * With the default `publicPrefix` and `privatePrefix`:
 * 
 * ```ts
 * import { env } from '$env/dynamic/public';
 * console.log(env.ENVIRONMENT); // => undefined, not public
 * console.log(env.PUBLIC_BASE_URL); // => "http://example.com"
 * ```
 * 
 * ```
 * 
 * ```
 */
declare module '$env/dynamic/public' {
	export const env: {
		[key: `PUBLIC_${string}`]: string | undefined;
	}
}
