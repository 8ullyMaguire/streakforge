import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';

export default defineConfig({
	// No @sveltejs/enhanced-img. It was registered but no component used
	// `<enhanced:img>`, and it cannot help here anyway: the only three <img>
	// tags take runtime URLs (a user's avatar_url, or a generated DiceBear
	// identicon), so there is no build-time asset for it to optimize. What it
	// did do was pull in vite-imagetools -> sharp, which has no prebuilt binary
	// for this platform and fails to compile under node-gyp. That made
	// `npm ci` fail outright, so the whole frontend -- build, vitest,
	// svelte-check -- was unrunnable.
	//
	// If build-time image optimization is ever wanted, re-add the plugin and
	// pin a sharp version with a prebuilt linux-x64 binary, rather than
	// relying on a source build.
	plugins: [sveltekit()],
	server: {
		proxy: {
			'/api': {
				target: 'http://127.0.0.1:8787',
				changeOrigin: true
			}
		},
		port: 5173
	}
});
