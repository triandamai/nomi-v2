<script lang="ts">
	// Admin → Configuration: the sections, each on a page of its own. Every row says how many of
	// its settings hold a value (saved here or from the environment).
	import { m } from '$lib/paraglide/messages';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { adminApi } from '$lib/adminApi';
	import { ORDER, SECTIONS, type Section } from './sections';

	let sections = $state<Section[] | null>(null);
	let loadError = $state<string | null>(null);

	async function load() {
		try {
			sections = (await adminApi<{ sections: Section[] }>('configuration')).sections;
		} catch (e) {
			loadError = (e as Error).message;
		}
	}

	$effect(() => {
		void load();
	});

	function filled(name: string): string | null {
		const section = sections?.find((s) => s.name === name);
		if (!section) return null;
		return m.config_fields_set({ set: section.fields.filter((f) => f.set).length, total: section.fields.length });
	}
</script>

<PageHeader title={m.admin_configuration()} lede={m.config_lede()} agent="coding" />

{#if loadError}
	<p class="error" role="alert">{loadError}</p>
{/if}

<ul class="list">
	{#each ORDER as name (name)}
		{@const count = filled(name)}
		<li>
			<a class="row" href="/admin/configuration/{name}">
				<span class="row__text">
					<span class="row__title">{SECTIONS[name].title()}</span>
					<span class="row__lede">{SECTIONS[name].lede()}</span>
				</span>
				{#if count}<span class="row__count">{count}</span>{/if}
			</a>
		</li>
	{/each}
</ul>

<style>
	.list {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(min(100%, 320px), 1fr));
		gap: 12px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.row {
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		gap: 12px;
		height: 100%;
		min-height: 44px;
		padding: 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
	}
	.row:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	.row:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.row__text {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.row__title {
		font-weight: 700;
	}
	.row__lede,
	.row__count {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.error {
		margin: 0 0 12px;
		color: var(--md-sys-color-error);
		overflow-wrap: anywhere;
	}
</style>
