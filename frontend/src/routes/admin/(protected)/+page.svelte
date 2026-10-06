<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import IconAgents from '$lib/components/icons/IconAgents.svelte';
	import IconChip from '$lib/components/icons/IconChip.svelte';
	import IconMemory from '$lib/components/icons/IconMemory.svelte';
	import IconPerson from '$lib/components/icons/IconPerson.svelte';
	import IconSparkle from '$lib/components/icons/IconSparkle.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const compact = new Intl.NumberFormat(getLocale(), { notation: 'compact', maximumFractionDigits: 1 });
	const full = new Intl.NumberFormat(getLocale());

	const stats = $derived([
		{ label: m.admin_people(), value: data.stats.total_users, note: m.admin_people_note() },
		{ label: m.admin_tokens_today(), value: data.stats.tokens_today, note: m.admin_tokens_today_note() },
		{ label: m.admin_tokens_all(), value: data.stats.tokens_all_time, note: m.admin_tokens_all_note() },
	]);
	const running = $derived(data.stats.running_agents);

	const AREAS = [
		{ href: '/admin/agents', title: m.admin_live_agents(), body: m.admin_area_agents(), icon: IconAgents, users: false },
		{ href: '/admin/dynamic-agents', title: m.admin_custom_agents(), body: m.admin_area_custom(), icon: IconSparkle, users: false },
		{ href: '/admin/settings/llm', title: m.admin_models(), body: m.admin_area_models(), icon: IconChip, users: false },
		{ href: '/admin/settings/embedding', title: m.admin_embeddings(), body: m.admin_area_embeddings(), icon: IconMemory, users: false },
		{ href: '/admin/users', title: m.admin_users(), body: m.admin_area_users(), icon: IconPerson, users: true },
	];
</script>

<PageHeader title={m.admin_overview()} lede={m.admin_overview_lede()} agent="supervisor" />

<section class="stats" aria-label={m.admin_at_glance()}>
	<div class="stat stat--live" data-active={running > 0}>
		<div class="stat__top">
			<span class="nomi-meta">{m.admin_running()}</span>
			<AgentShape agent="nomi" face size={44} working={running > 0} />
		</div>
		<span class="stat__value">{full.format(running)}</span>
		<span class="stat__note">{running === 1 ? m.admin_agent_working() : m.admin_agents_working()}</span>
	</div>
	{#each stats as stat (stat.label)}
		<div class="stat">
			<span class="nomi-meta">{stat.label}</span>
			<span class="stat__value" title={full.format(stat.value)}>{compact.format(stat.value)}</span>
			<span class="stat__note">{stat.note}</span>
		</div>
	{/each}
</section>

<section class="areas" aria-labelledby="areas-title">
	<h2 id="areas-title" class="section-label">{m.admin_manage()}</h2>
	<ul class="areas__grid">
		{#each AREAS.filter((a) => (a.users ? data.canViewUsers : data.canManageSystemConfig)) as area (area.href)}
			<li>
				<a class="area" href={area.href}>
					<span class="area__icon"><area.icon size={24} /></span>
					<span class="area__title">{area.title}</span>
					<span class="area__body">{area.body}</span>
				</a>
			</li>
		{/each}
	</ul>
</section>

<style>
	.stats {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(100%, 160px), 1fr));
		gap: 12px;
	}
	.stat {
		display: flex;
		flex-direction: column;
		gap: 6px;
		min-width: 0;
		padding: 20px 22px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.stat--live[data-active='true'] {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.stat__top {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		margin: -6px 0 -6px;
	}
	.stat__value {
		font-family: var(--md-ref-typeface-brand);
		font-size: 2.75rem;
		font-weight: 800;
		line-height: 1;
		letter-spacing: -0.03em;
		font-variant-numeric: tabular-nums;
	}
	.stat__note {
		font-size: 0.875rem;
		opacity: 0.75;
	}
	.section-label {
		margin: 0 4px 12px;
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		font-weight: 400;
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	.areas__grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(min(100%, 260px), 1fr));
		gap: 12px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.area {
		display: grid;
		grid-template-columns: auto 1fr;
		grid-template-rows: auto auto;
		column-gap: 14px;
		row-gap: 4px;
		height: 100%;
		box-sizing: border-box;
		padding: 18px 20px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-fast);
	}
	.area:hover {
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-low);
	}
	.area__icon {
		grid-row: span 2;
		display: grid;
		place-items: center;
		width: 48px;
		height: 48px;
		border-radius: 16px;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.area__title {
		align-self: end;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.125rem;
		font-weight: 700;
	}
	.area__body {
		font-size: 0.875rem;
		line-height: 1.45;
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
