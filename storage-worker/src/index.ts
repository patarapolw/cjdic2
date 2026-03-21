/**
 * Welcome to Cloudflare Workers! This is your first worker.
 *
 * - Run `npm run dev` in your terminal to start a development server
 * - Open a browser tab at http://localhost:8787/ to see your worker in action
 * - Run `npm run deploy` to publish your worker
 *
 * Bind resources to your worker in `wrangler.jsonc`. After adding bindings, a type definition for the
 * `Env` object can be regenerated with `npm run cf-typegen`.
 *
 * Learn more at https://developers.cloudflare.com/workers/
 */

export default {
	async fetch(request, env) {
		const url = new URL(request.url);

		// List files: GET /files
		if (url.pathname === '/files') {
			const list = await env.BUCKET.list();
			const files = list.objects
				.filter((o) => o.size)
				.map((o) => ({
					key: o.key,
					size: o.size,
					uploaded: o.uploaded,
					url: `${url.origin}/download/${o.key}`,
				}));
			return Response.json(files);
		}

		// Download: GET /download/filename.zip
		if (url.pathname.startsWith('/download/')) {
			const key = url.pathname.replace('/download/', '');
			const o = await env.BUCKET.get(key);
			if (!o || !o.size) return new Response('Not found', { status: 404 });
			return new Response(o.body, {
				headers: { 'Content-Disposition': `attachment; filename="${key}"` },
			});
		}

		return new Response('Not found', { status: 404 });
	},
} satisfies ExportedHandler<Env>;
