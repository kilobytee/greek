import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
	plugins: [
		sveltekit({
			compilerOptions: {
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},
			adapter: adapter({ fallback: 'index.html', strict: false })
		})
	],
	server: {
		port: 1420,
		strictPort: true,
		host: host || false,
		hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
		watch: {
			ignored: ['**/src-tauri/**']
		}
	},
	clearScreen: false,
	envPrefix: ['VITE_', 'TAURI_'],
	build: {
		target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
		minify: !process.env.TAURI_ENV_DEBUG,
		sourcemap: !!process.env.TAURI_ENV_DEBUG
	}
});
