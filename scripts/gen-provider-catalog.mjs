#!/usr/bin/env node
// Builds the provider catalog Nomi offers under Providers (Admin and personal) from models.dev
// (MIT, https://models.dev) plus scripts/provider-catalog.overrides.json, and fetches each
// provider's logo. Run by hand when the catalog should be refreshed; the output is committed, so
// builds and CI never reach the network.
//
//   node scripts/gen-provider-catalog.mjs                 # downloads https://models.dev/api.json
//   node scripts/gen-provider-catalog.mjs path/api.json   # or reads a saved copy
//   node scripts/gen-provider-catalog.mjs --no-logos      # skip the logo download
//
// Writes backend/crates/nomi-llm/catalog/providers.json and frontend/static/provider-logos/*.svg.

import { existsSync } from 'node:fs';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const CATALOG_PATH = join(root, 'backend/crates/nomi-llm/catalog/providers.json');
const LOGO_DIR = join(root, 'frontend/static/provider-logos');

const args = process.argv.slice(2);
const withLogos = !args.includes('--no-logos');
const source = args.find((arg) => !arg.startsWith('--'));

const overrides = JSON.parse(await readFile(join(root, 'scripts/provider-catalog.overrides.json'), 'utf8'));
const modelsDev = source
	? JSON.parse(await readFile(source, 'utf8'))
	: await (await fetch('https://models.dev/api.json')).json();

/** The runtime Nomi uses for a provider and where it lives, or null when Nomi can't run it. */
function runtimeFor(id, provider) {
	if (overrides.kinds[id]) return { kind: overrides.kinds[id], baseUrl: provider.api ?? null };
	const listed = overrides.openai_compatible_base_urls[id];
	if (listed) return { kind: 'openai_compatible', baseUrl: listed };
	const api = provider.api ?? '';
	// Templated URLs (an account id, a host) need more than a key; not offered yet. Plain http is
	// an app on someone's own computer (LM Studio, …), which Nomi's server can't reach.
	if (!api.startsWith('https://') || api.includes('${') || api.includes('{')) return null;
	switch (provider.npm) {
		case '@ai-sdk/openai-compatible':
		case '@ai-sdk/openai':
			return { kind: 'openai_compatible', baseUrl: api };
		case '@ai-sdk/anthropic':
			// Nomi's Anthropic client adds /v1/messages itself.
			return { kind: 'anthropic', baseUrl: api.replace(/\/v1\/?$/, '') };
		default:
			return null;
	}
}

function toModel(id, model) {
	const cost = model.cost ?? {};
	return {
		id,
		name: model.name ?? id,
		input: model.modalities?.input ?? ['text'],
		cost: {
			input: cost.input ?? null,
			output: cost.output ?? null,
			cache_read: cost.cache_read ?? null,
		},
		context: model.limit?.context ?? null,
	};
}

const providers = [];
const skipped = [];
for (const [id, provider] of Object.entries(modelsDev)) {
	if (overrides.excluded[id]) {
		skipped.push(`${id}: ${overrides.excluded[id]}`);
		continue;
	}
	const runtime = runtimeFor(id, provider);
	if (!runtime) {
		skipped.push(`${id}: no runtime for ${provider.npm}`);
		continue;
	}
	// Nomi's crew works through tools, so only models that can call them are offered.
	const models = Object.entries(provider.models ?? {})
		.filter(([, model]) => model.tool_call)
		.map(([modelId, model]) => toModel(modelId, model))
		.sort((a, b) => a.name.localeCompare(b.name));
	if (models.length === 0) {
		skipped.push(`${id}: no models that can use tools`);
		continue;
	}
	const popularIndex = overrides.popular.indexOf(id);
	providers.push({
		id,
		name: provider.name,
		description: overrides.descriptions[id] ?? '',
		category: popularIndex >= 0 ? 'popular' : 'other',
		recommended: overrides.recommended.includes(id),
		auth_type: 'api_key',
		kind: runtime.kind,
		default_base_url: runtime.baseUrl,
		doc_url: provider.doc ?? null,
		models,
	});
}

const order = (p) => (p.category === 'popular' ? overrides.popular.indexOf(p.id) : overrides.popular.length);
providers.sort((a, b) => order(a) - order(b) || a.name.localeCompare(b.name));

// Any OpenAI-compatible endpoint the catalog doesn't list: base URL, headers and models are the
// person's own.
providers.push({
	id: 'custom',
	name: 'Custom OpenAI-compatible provider',
	description: 'Any endpoint that speaks the OpenAI API',
	category: 'custom',
	recommended: false,
	auth_type: 'custom_endpoint',
	kind: 'openai_compatible',
	default_base_url: null,
	doc_url: null,
	models: [],
});

const missingPopular = overrides.popular.filter((id) => !providers.some((p) => p.id === id));
if (missingPopular.length) throw new Error(`popular providers missing from the catalog: ${missingPopular.join(', ')}`);


if (withLogos) {
	await mkdir(LOGO_DIR, { recursive: true });
	const unsafe = /<script|<foreignObject|\son[a-z]+\s*=|(?:xlink:)?href\s*=\s*["'](?!#)|url\(\s*["']?(?!#)/i;
	let saved = 0;
	const failed = [];
	const queue = providers.filter((p) => p.id !== 'custom');
	await Promise.all(
		Array.from({ length: 8 }, async () => {
			for (let p = queue.shift(); p; p = queue.shift()) {
				try {
					const response = await fetch(`https://models.dev/logos/${p.id}.svg`);
					const svg = await response.text();
					if (!response.ok || !svg.trimStart().startsWith('<svg') || unsafe.test(svg)) throw new Error(`status ${response.status}`);
					await writeFile(join(LOGO_DIR, `${p.id}.svg`), svg);
					saved++;
				} catch (error) {
					failed.push(`${p.id} (${error.message})`);
				}
			}
		}),
	);
	console.log(`${saved} logos -> ${LOGO_DIR}${failed.length ? `; no logo for: ${failed.join(', ')}` : ''}`);
}

// Written last: whether each provider has a logo is only known once they've been fetched.
for (const provider of providers) provider.has_logo = provider.id !== 'custom' && existsSync(join(LOGO_DIR, `${provider.id}.svg`));
await mkdir(dirname(CATALOG_PATH), { recursive: true });
await writeFile(CATALOG_PATH, `${JSON.stringify({ source: 'https://models.dev (MIT)', providers }, null, '\t')}\n`);
console.log(`${providers.length} providers (${providers.reduce((n, p) => n + p.models.length, 0)} models) -> ${CATALOG_PATH}`);
console.log(`${skipped.length} skipped:\n  ${skipped.join('\n  ')}`);
