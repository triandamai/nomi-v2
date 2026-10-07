<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { deserialize, enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import Switch from '$lib/components/m3/Switch.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import IconSearch from '$lib/components/icons/IconSearch.svelte';
	import type { PageData } from './$types';
	import type { ReadPageSettings, SearchHit, ToolEntry, ToolGroup, WebSearchSettings } from './+page.server';

	let { data }: { data: PageData } = $props();

	const PROVIDER_LABELS: Record<string, () => string> = {
		tavily: m.tools_provider_tavily,
		serper: m.tools_provider_serper,
		brave: m.tools_provider_brave,
		searxng: m.tools_provider_searxng,
	};
	const PROVIDER_HINTS: Record<string, () => string> = {
		tavily: m.tools_provider_tavily_hint,
		serper: m.tools_provider_serper_hint,
		brave: m.tools_provider_brave_hint,
		searxng: m.tools_provider_searxng_hint,
	};
	const providerLabel = (id: string) => PROVIDER_LABELS[id]?.() ?? id;
	const PROVIDER_NAMES: Record<string, string> = { tavily: 'Tavily', serper: 'Serper', brave: 'Brave Search', searxng: 'SearXNG' };

	// Switches flip right away; a save that fails puts them back.
	let overrides = $state<Record<string, boolean>>({});
	let toggleError = $state<string | null>(null);

	const allTools = $derived((data.groups ?? []).flatMap((g) => g.tools));
	const webSearch = $derived(allTools.find((t) => t.name === 'web_search'));
	const searchSettings = $derived(webSearch?.settings as WebSearchSettings | undefined);
	const enabled = (tool: ToolEntry) => tool.required || (overrides[tool.name] ?? tool.enabled);

	async function toggle(tool: ToolEntry, on: boolean) {
		overrides[tool.name] = on;
		toggleError = null;
		const body = new FormData();
		body.set('name', tool.name);
		body.set('enabled', String(on));
		const result = deserialize(await (await fetch('?/toggle', { method: 'POST', body })).text());
		if (result.type !== 'success') {
			overrides[tool.name] = !on;
			toggleError = m.tools_toggle_failed({ tool: tool.name });
		}
	}

	function groupTitle(group: ToolGroup) {
		if (group.kind === 'crew') return m.tools_group_crew();
		if (group.kind === 'custom') return m.tools_group_custom();
		return group.label;
	}
	function groupLede(group: ToolGroup) {
		if (group.kind === 'crew') return m.tools_group_crew_lede();
		if (group.kind === 'custom') return m.tools_group_custom_lede();
		return m.tools_group_agent_lede({ agent: group.label });
	}

	// The web search sheet.
	let searchOpen = $state(false);
	let provider = $state('tavily');
	let apiKey = $state('');
	let clearKey = $state(false);
	let baseUrl = $state('');
	let maxResults = $state('5');
	let saveError = $state<string | null>(null);
	let saving = $state(false);
	let saved = $state(false);
	let testQuery = $state('');
	let testing = $state(false);
	type TestResult = { ok: boolean; provider: string; results: SearchHit[]; error: string | null };
	let testResult = $state<TestResult | null>(null);

	const chosen = $derived(searchSettings?.providers.find((p) => p.id === provider));

	function openSearch() {
		provider = searchSettings?.provider ?? 'tavily';
		apiKey = '';
		clearKey = false;
		baseUrl = searchSettings?.base_url ?? '';
		maxResults = String(searchSettings?.max_results ?? 5);
		saveError = null;
		saved = false;
		testResult = null;
		searchOpen = true;
	}

	function keyPlaceholder() {
		if (!chosen) return m.tools_key_missing();
		if (chosen.key_source === 'saved') return m.tools_key_saved();
		if (chosen.key_source === 'env') return m.tools_key_env({ env: chosen.env_var });
		return m.tools_key_missing();
	}

	async function runTest() {
		testing = true;
		testResult = null;
		const body = new FormData();
		body.set('query', testQuery);
		const result = deserialize(await (await fetch('?/test', { method: 'POST', body })).text());
		testing = false;
		if (result.type === 'success' && result.data?.test) {
			testResult = result.data.test as TestResult;
		} else {
			testResult = {
				ok: false,
				provider,
				results: [],
				error: (result.type === 'failure' && (result.data?.error as string)) || m.err_save_tool(),
			};
		}
	}

	// The page reader's sheet.
	let readOpen = $state(false);
	let maxChars = $state('12000');
	let readError = $state<string | null>(null);

	function openSettings(tool: ToolEntry) {
		if (tool.name === 'web_search') return openSearch();
		maxChars = String((tool.settings as ReadPageSettings | null)?.max_chars ?? 12000);
		readError = null;
		readOpen = true;
	}
</script>

<PageHeader title={m.admin_tools()} lede={m.tools_lede()} agent="supervisor" />

{#if !data.groups}
	<p class="md-body-large" style="color: var(--md-sys-color-error)">{m.tools_load_failed()}</p>
{:else}
	{#if webSearch}
		{@const on = enabled(webSearch)}
		<section class="hero" aria-label={m.tools_web_title()}>
			<span class="hero__icon"><IconSearch size={26} /></span>
			<div class="hero__text">
				<span class="nomi-meta">{m.tools_web_title()}</span>
				<h2 class="hero__name">{PROVIDER_NAMES[searchSettings?.provider ?? 'tavily'] ?? searchSettings?.provider}</h2>
				<p class="hero__status" class:hero__status--warn={on && !webSearch.ready}>
					{#if !on}
						{m.tools_web_off()}
					{:else if webSearch.ready}
						{m.tools_web_ready()}
					{:else}
						{m.tools_web_needs_key()}
					{/if}
				</p>
				<p class="hero__hint">{m.tools_web_hint()}</p>
			</div>
			<Button type="button" variant={webSearch.ready ? 'tonal' : 'filled'} onclick={openSearch}>
				{webSearch.ready ? m.tools_change() : m.tools_set_up()}
			</Button>
		</section>
	{/if}

	{#if toggleError}
		<p class="md-body-medium" role="alert" style="color: var(--md-sys-color-error)">{toggleError}</p>
	{/if}

	{#each data.groups as group (group.key)}
		<section class="group" aria-labelledby="group-{group.key}">
			<header class="group__head">
				<h2 class="group__title" id="group-{group.key}">{groupTitle(group)}</h2>
				<span class="group__count">{group.tools.length}</span>
			</header>
			<p class="group__lede">{groupLede(group)}</p>
			<ul class="tools">
				{#each group.tools as tool (tool.name)}
					{@const on = enabled(tool)}
					<li class="tool" class:tool--off={!on}>
						<div class="tool__text">
							<div class="tool__name-row">
								<code class="tool__name">{tool.name}</code>
								{#if tool.required}
									<span class="badge">{m.tools_required()}</span>
								{:else if on && !tool.ready}
									<span class="badge badge--warn">{m.tools_needs_key()}</span>
								{/if}
							</div>
							<p class="tool__desc">{tool.description}</p>
							{#if tool.used_by.length > 1}
								<p class="tool__meta">{m.tools_also_used_by({ agents: tool.used_by.slice(1).join(', ') })}</p>
							{/if}
						</div>
						<div class="tool__actions">
							{#if tool.configurable}
								<Button type="button" variant="text" onclick={() => openSettings(tool)}>{m.tools_settings()}</Button>
							{/if}
							<Switch
								checked={on}
								disabled={tool.required}
								aria-label={m.tools_toggle({ tool: tool.name })}
								onchange={(value) => toggle(tool, value)}
							/>
						</div>
					</li>
				{/each}
			</ul>
		</section>
	{/each}
{/if}

<BottomSheet bind:open={searchOpen}>
	<h2 class="sheet-title">{m.tools_web_title()}</h2>
	{#if saveError}
		<p class="md-body-medium mt-2" role="alert" style="color: var(--md-sys-color-error)">{saveError}</p>
	{/if}
	<form
		method="POST"
		action="?/saveSearch"
		class="mt-4 flex flex-col gap-3"
		use:enhance={() => {
			saving = true;
			saved = false;
			saveError = null;
			return async ({ result }) => {
				saving = false;
				if (result.type === 'success') {
					await invalidateAll();
					apiKey = '';
					clearKey = false;
					saved = true;
				} else if (result.type === 'failure') {
					saveError = (result.data?.error as string) || m.err_save_tool();
				}
			};
		}}
	>
		<Select
			label={m.tools_provider()}
			name="provider"
			bind:value={provider}
			options={(searchSettings?.providers ?? []).map((p) => ({ value: p.id, label: providerLabel(p.id) }))}
		/>
		<p class="hint">{PROVIDER_HINTS[provider]?.()}</p>

		{#if chosen?.needs_key}
			<TextField
				id="api_key"
				name="api_key"
				type="password"
				autocomplete="off"
				label={m.key_api_key()}
				bind:value={apiKey}
				placeholder={keyPlaceholder()}
			/>
			{#if chosen.key_source === 'saved'}
				<label class="check">
					<input type="checkbox" bind:checked={clearKey} />
					<span>{m.tools_clear_key()}</span>
				</label>
				<input type="hidden" name="clear_api_key" value={String(clearKey)} />
			{/if}
		{/if}
		<TextField
			id="base_url"
			name="base_url"
			type="url"
			label={provider === 'searxng' ? m.tools_searxng_url() : m.tools_base_url()}
			placeholder={provider === 'searxng' ? 'https://search.example.com' : ''}
			bind:value={baseUrl}
		/>
		<TextField
			id="max_results"
			name="max_results"
			type="number"
			min="1"
			max={searchSettings?.max_results_limit ?? 10}
			label={m.tools_max_results()}
			supportingText={m.tools_max_results_hint({ max: String(searchSettings?.max_results_limit ?? 10) })}
			bind:value={maxResults}
		/>
		<div class="flex flex-wrap gap-2 pt-2">
			<Button type="submit" variant="filled" disabled={saving}>{m.common_save()}</Button>
			<Button type="button" variant="outlined" onclick={() => (searchOpen = false)}>{m.common_cancel()}</Button>
			{#if saved}<span class="saved" role="status">{m.tools_saved()}</span>{/if}
		</div>
	</form>

	<div class="test">
		<TextField id="test_query" label={m.tools_test_query()} placeholder="latest news" bind:value={testQuery} supportingText={m.tools_test_hint()} />
		<Button type="button" variant="tonal" class="w-fit" disabled={testing} onclick={runTest}>
			{testing ? m.tools_testing() : m.tools_test()}
		</Button>
		{#if testResult}
			{#if testResult.ok}
				<p class="test__ok">{m.tools_test_ok({ count: String(testResult.results.length), provider: providerLabel(testResult.provider) })}</p>
				<ol class="hits">
					{#each testResult.results as hit (hit.url)}
						<li>
							<a href={hit.url} target="_blank" rel="noopener noreferrer">{hit.title || hit.url}</a>
							{#if hit.snippet}<p>{hit.snippet}</p>{/if}
						</li>
					{/each}
				</ol>
			{:else}
				<p class="test__error" role="alert">{testResult.error}</p>
			{/if}
		{/if}
	</div>
</BottomSheet>

<BottomSheet bind:open={readOpen}>
	<h2 class="sheet-title">read_web_page</h2>
	{#if readError}
		<p class="md-body-medium mt-2" role="alert" style="color: var(--md-sys-color-error)">{readError}</p>
	{/if}
	<form
		method="POST"
		action="?/saveReadPage"
		class="mt-4 flex flex-col gap-3"
		use:enhance={() => {
			readError = null;
			return async ({ result }) => {
				if (result.type === 'success') {
					await invalidateAll();
					readOpen = false;
				} else if (result.type === 'failure') {
					readError = (result.data?.error as string) || m.err_save_tool();
				}
			};
		}}
	>
		<TextField
			id="max_chars"
			name="max_chars"
			type="number"
			min="1000"
			max="50000"
			step="1000"
			label={m.tools_max_chars()}
			supportingText={m.tools_max_chars_hint()}
			bind:value={maxChars}
		/>
		<div class="flex gap-2 pt-2">
			<Button type="submit" variant="filled">{m.common_save()}</Button>
			<Button type="button" variant="outlined" onclick={() => (readOpen = false)}>{m.common_cancel()}</Button>
		</div>
	</form>
</BottomSheet>

<style>
	.hero {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 16px 20px;
		padding: 22px 24px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.hero__icon {
		display: grid;
		flex: none;
		place-items: center;
		width: 56px;
		height: 56px;
		border-radius: 18px;
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.hero__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 200px;
	}
	.hero__name {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.375rem;
		font-weight: 700;
	}
	.hero__status {
		margin: 0;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.hero__status--warn {
		color: var(--md-sys-color-error);
	}
	.hero__hint {
		margin: 6px 0 0;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.group {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.group__head {
		display: flex;
		align-items: baseline;
		gap: 10px;
	}
	.group__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.25rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.group__count {
		padding: 1px 8px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
	}
	.group__lede {
		margin: 0 0 6px;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.tools {
		display: flex;
		flex-direction: column;
		gap: 2px;
		margin: 0;
		padding: 0;
		list-style: none;
		border-radius: var(--md-sys-shape-corner-extra-large);
		overflow: hidden;
	}
	.tool {
		display: flex;
		align-items: center;
		gap: 12px 16px;
		padding: 14px 18px;
		background: var(--md-sys-color-surface-container-lowest);
	}
	.tool__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
	}
	.tool--off .tool__text {
		opacity: 0.6;
	}
	.tool__name-row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
	}
	.tool__name {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
		overflow-wrap: anywhere;
	}
	.tool__desc {
		display: -webkit-box;
		margin: 0;
		overflow: hidden;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
		-webkit-box-orient: vertical;
		-webkit-line-clamp: 2;
		line-clamp: 2;
	}
	.tool__meta {
		margin: 0;
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.tool__actions {
		display: flex;
		flex: none;
		align-items: center;
		gap: 8px;
	}
	.badge {
		padding: 1px 8px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font-size: 0.6875rem;
		font-weight: 600;
		letter-spacing: 0.02em;
	}
	.badge--warn {
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}
	@media (max-width: 599px) {
		.hero {
			align-items: flex-start;
			padding: 20px;
		}
		.hero__icon {
			width: 48px;
			height: 48px;
			border-radius: 16px;
		}
		.tool {
			align-items: flex-start;
			padding: 14px 16px;
		}
		/* The switch stays beside the name; the settings button drops under it. */
		.tool__actions {
			flex-direction: column-reverse;
			align-items: flex-end;
			gap: 4px;
		}
	}
	.sheet-title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.5rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.hint {
		margin: -4px 0 4px;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.saved {
		align-self: center;
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--md-sys-color-primary);
	}
	.check {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.test {
		display: flex;
		flex-direction: column;
		gap: 10px;
		margin-top: 24px;
		padding-top: 20px;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.test__ok {
		margin: 0;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}
	.test__error {
		margin: 0;
		color: var(--md-sys-color-error);
		overflow-wrap: anywhere;
	}
	.hits {
		display: flex;
		flex-direction: column;
		gap: 10px;
		margin: 0;
		padding-left: 20px;
		font-size: 0.875rem;
	}
	.hits a {
		color: var(--md-sys-color-primary);
		overflow-wrap: anywhere;
	}
	.hits p {
		margin: 2px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
