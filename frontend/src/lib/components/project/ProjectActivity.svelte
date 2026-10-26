<script lang="ts">
	// Members → Activity: who has the project open now, and what happens in it, newest first
	// (backend routes::project_activity). Refreshes when the project's live topic says something
	// changed, and now and then while open, since prompts and builds don't announce themselves there.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import Avatar from '$lib/components/m3/Avatar.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import StatusSquare from '$lib/components/project/StatusSquare.svelte';
	import { environmentName, phaseTitle } from '$lib/components/project/deployLabels';
	import { ago, projectApi } from '$lib/projectApi';
	import type { ProjectDeploy } from '$lib/projectDeploy.svelte';
	import type { ProjectPresence } from '$lib/projectPresence.svelte';

	const PAGE_SIZE = 50;
	const REFRESH_MS = 15_000;

	interface ActivityItem {
		kind: string;
		at: string;
		actor: { name: string | null; avatar_url: string | null; agent: string | null } | null;
		text?: string;
		version?: number;
		environment?: string;
		status?: string;
		phase?: string;
		step?: string;
		files?: number;
		rollback_of_version?: number;
	}

	let { projectId, presence = null, deploy = null }: { projectId: string; presence?: ProjectPresence | null; deploy?: ProjectDeploy | null } = $props();

	const locale = getLocale();
	let items = $state<ActivityItem[]>([]);
	let hasMore = $state(false);
	let error = $state<string | null>(null);
	let loaded = $state(false);

	/** The newest page again, keeping anything older already loaded below it. */
	async function refresh() {
		try {
			const page = await projectApi<{ items: ActivityItem[]; has_more: boolean }>(projectId, `activity?limit=${PAGE_SIZE}`);
			const oldest = page.items.at(-1)?.at;
			const older = oldest ? items.filter((item) => item.at < oldest) : [];
			items = [...page.items, ...older];
			if (older.length === 0) hasMore = page.has_more;
			error = null;
		} catch (e) {
			error = (e as Error).message;
		} finally {
			loaded = true;
		}
	}

	async function more() {
		const before = items.at(-1)?.at;
		if (!before) return;
		try {
			const page = await projectApi<{ items: ActivityItem[]; has_more: boolean }>(projectId, `activity?limit=${PAGE_SIZE}&before=${encodeURIComponent(before)}`);
			items = [...items, ...page.items];
			hasMore = page.has_more;
		} catch (e) {
			error = (e as Error).message;
		}
	}

	$effect(() => {
		void refresh();
		const timer = setInterval(() => !document.hidden && void refresh(), REFRESH_MS);
		const unlisten = deploy?.onEvent((data) => {
			if (!data.includes('"Typing"')) void refresh();
		});
		return () => {
			clearInterval(timer);
			unlisten?.();
		};
	});

	function who(item: ActivityItem): string {
		return item.actor?.agent ?? item.actor?.name ?? m.pj_act_someone();
	}

	function sentence(item: ActivityItem): string {
		const name = who(item);
		const agent = item.actor?.agent ?? name;
		const env = environmentName(item.environment ?? 'production');
		const version = item.version ?? 0;
		switch (item.kind) {
			case 'prompt':
				return m.pj_act_prompt({ name, text: item.text ?? '' });
			case 'build_started':
				return m.pj_act_build_started({ agent });
			case 'build_finished':
				return m.pj_act_build_finished({ agent });
			case 'build_failed':
				return m.pj_act_build_failed({ agent });
			case 'build_stopped':
				return m.pj_act_build_stopped({ agent });
			case 'files_changed':
				return item.files === 1 ? m.pj_act_files_one({ agent }) : m.pj_act_files({ agent, count: item.files ?? 0 });
			case 'publishing':
				return m.pj_act_publishing({ name, version, env });
			case 'published':
				return m.pj_act_published({ name, version, env });
			case 'publish_failed':
				return item.phase && item.step
					? m.pj_act_publish_failed({ name, version, env, phase: phaseTitle(item.phase), step: item.step })
					: m.pj_act_publish_failed_plain({ name, version, env });
			case 'rolled_back':
				return m.pj_act_rolled_back({ name, env, version: item.rollback_of_version ?? version });
			case 'joined':
				return m.pj_act_joined({ name });
			default:
				return item.kind;
		}
	}

	/** Builds and publishes show how they went; the rest show who. */
	function status(item: ActivityItem): string | null {
		if (item.kind === 'publish_failed' || item.kind === 'build_failed') return 'failed';
		if (item.kind === 'published' || item.kind === 'build_finished') return 'succeeded';
		// A build's start is history once it finished: only a publish still going reads as running.
		if (item.kind === 'publishing') return 'running';
		if (item.kind === 'rolled_back') return item.status === 'failed' ? 'failed' : 'succeeded';
		return null;
	}
</script>

<section class="here" aria-label={m.pj_here_now()}>
	<h3 class="title">{m.pj_here_now()}</h3>
	{#if presence && presence.people.length > 0}
		<ul class="faces">
			{#each presence.people as person (person.user_id)}
				<li class="face"><Avatar name={person.name} avatarUrl={person.avatar_url} size={32} /><span>{person.name}</span></li>
			{/each}
		</ul>
	{:else}
		<p class="muted">{m.pj_act_only_you()}</p>
	{/if}
</section>

<section aria-label={m.pj_act_tab_activity()}>
	{#if error}<p class="error" role="alert">{error}</p>{/if}
	{#if loaded && items.length === 0 && !error}
		<p class="muted">{m.pj_act_empty()}</p>
	{:else if items.length > 0}
		<ul class="feed">
			{#each items as item, i (`${item.kind}-${item.at}-${i}`)}
				{@const tone = status(item)}
				<li class="row">
					{#if tone}
						<StatusSquare status={tone} size={32} />
					{:else}
						<Avatar name={who(item)} avatarUrl={item.actor?.agent ? null : (item.actor?.avatar_url ?? null)} size={32} />
					{/if}
					<span class="row__text">{sentence(item)}</span>
					<time class="row__when" datetime={item.at}>{ago(item.at, locale)}</time>
				</li>
			{/each}
		</ul>
		{#if hasMore}
			<div class="more"><Button variant="text" size="m" onclick={more}>{m.pj_act_more()}</Button></div>
		{/if}
	{/if}
</section>

<style>
	.title {
		margin: 0 0 10px;
		color: var(--md-sys-color-on-surface);
		font-size: 0.9375rem;
		font-weight: 700;
	}
	.here {
		margin-bottom: 16px;
		padding: 14px 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.faces {
		display: flex;
		flex-wrap: wrap;
		gap: 10px 18px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.face {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		color: var(--md-sys-color-on-surface);
		font-size: 0.875rem;
	}
	.muted {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.error {
		color: var(--md-sys-color-error);
		font-size: 0.875rem;
	}
	.feed {
		margin: 0;
		padding: 0 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
		list-style: none;
	}
	.row {
		display: grid;
		grid-template-columns: auto 1fr auto;
		gap: 12px;
		align-items: center;
		min-height: 56px;
		padding: 10px 0;
		border-top: 1px solid var(--md-sys-color-surface-container-low);
	}
	.row:first-child {
		border-top: 0;
	}
	.row__text {
		min-width: 0;
		color: var(--md-sys-color-on-surface);
		font-size: 0.875rem;
		line-height: 1.4;
		overflow-wrap: anywhere;
	}
	.row__when {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		white-space: nowrap;
	}
	.more {
		display: flex;
		justify-content: center;
		margin-top: 8px;
	}
</style>
