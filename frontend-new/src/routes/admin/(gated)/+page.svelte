<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { compact, full } from '$lib/admin/format';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import StatTile from '$lib/ds/admin/StatTile.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import Link from '$lib/ds/actions/Link.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';

	// Overview (dashboard: view): who is on Nomi, the crew at work, tokens, and the server-side
	// sandboxes projects run in, from GET /api/admin/dashboard. The real figures, shortened where long
	// with the exact one in the tile's title; a failure says so instead of showing zeros.
	let { data } = $props();

	const language = getLocale();
	const tile = (value: number) => ({ value: compact(value, language), exact: full(value, language) });
</script>

<PageHeader title={m.admin_page_overview()} intro={m.admin_overview_intro()} />

{#if data.dashboard.ok}
	{@const stats = data.dashboard.data}
	<section aria-label={m.admin_overview_glance()} class="flex flex-col gap-3">
		<SectionLabel label={m.admin_overview_glance()} />
		<div class="grid grid-cols-2 gap-3 lg:grid-cols-4">
			<StatTile label={m.admin_stat_running()} value={full(stats.running_agents, language)} note={m.admin_stat_running_note()} live={stats.running_agents > 0} />
			<StatTile label={m.admin_stat_people()} {...tile(stats.total_users)} note={m.admin_stat_people_note()} />
			<StatTile label={m.admin_stat_tokens_today()} {...tile(stats.tokens_today)} note={m.admin_stat_tokens_today_note()} />
			<StatTile label={m.admin_stat_tokens_all()} {...tile(stats.tokens_all_time)} note={m.admin_stat_tokens_all_note()} />
		</div>
		<p class="text-caption text-text-3">{m.admin_overview_tokens_note()}</p>
	</section>
	<section aria-label={m.admin_sandboxes()} class="flex flex-col gap-3" data-testid="sandbox-stats">
		<SectionLabel label={m.admin_sandboxes()} />
		<div class="grid grid-cols-2 gap-3 lg:grid-cols-4">
			<StatTile label={m.admin_stat_sandbox_running()} value={full(stats.sandbox.running, language)} note={m.admin_stat_sandbox_running_note()} live={stats.sandbox.running > 0} />
			<StatTile label={m.admin_stat_sandbox_starting()} value={full(stats.sandbox.starting, language)} note={m.admin_stat_sandbox_starting_note()} />
			<StatTile label={m.admin_stat_sandbox_error()} value={full(stats.sandbox.error, language)} note={m.admin_stat_sandbox_error_note()} />
			<StatTile label={m.admin_stat_sandbox_minutes()} {...tile(stats.sandbox.minutes_this_month)} note={m.admin_stat_sandbox_minutes_note()} />
		</div>
	</section>
	<section aria-label={m.admin_overview_more()} class="flex flex-col gap-3">
		<SectionLabel label={m.admin_overview_more()} />
		<div class="flex flex-wrap gap-x-6 gap-y-2">
			<Link href="/admin/agents" label={m.admin_overview_live_link()} />
			<Link href="/admin/usage" label={m.admin_overview_usage_link()} />
		</div>
	</section>
{:else}
	<LoadFailed title={m.admin_overview_failed()} problem={data.dashboard.problem} />
{/if}
