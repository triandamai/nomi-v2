<script lang="ts">
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import { crewByActivity, rosterKey, type CrewRosterMember } from '$lib/crew';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	// Every agent the app has, with what each is doing for you right now (/api/agents, loaded by
	// the app layout).
	const crew = $derived(crewByActivity(data.crew));
	const busy = $derived(crew.filter((m) => m.state !== 'idle'));
	const ready = $derived(crew.filter((m) => m.state === 'idle'));

	const BADGES: Record<CrewRosterMember['state'], string | null> = {
		working: 'Working',
		waiting: 'Needs you',
		done: 'Done',
		idle: null,
	};
</script>

<svelte:head>
	<title>Your crew · Nomi</title>
</svelte:head>

{#snippet card(member: CrewRosterMember)}
	{@const key = rosterKey(member.agent_type)}
	<li class="card" data-state={member.state}>
		<AgentShape agent={key} size={56} face={key === 'nomi'} working={member.state === 'working'} />
		<div class="card__text">
			<div class="card__top">
				<h3 class="card__name">{member.name}</h3>
				{#if member.is_dynamic}<span class="card__custom">Custom</span>{/if}
				{#if BADGES[member.state]}
					<span class="card__badge" data-state={member.state}>{BADGES[member.state]}</span>
				{/if}
			</div>
			<p class="card__role">{member.role}</p>
			{#if member.state !== 'idle' && member.status}
				<p class="card__status">{member.status}</p>
			{/if}
		</div>
	</li>
{/snippet}

<div class="crew-page">
	<div class="crew-page__inner">
		<header class="crew-page__head">
			<h1 class="md-display-small crew-page__title">Your crew</h1>
			<p class="md-body-large crew-page__lede">
				Everyone who works with Nomi. Ask in any chat: Nomi hands each job to the right one.
			</p>
		</header>

		{#if crew.length === 0}
			<p class="crew-page__empty">Couldn't load your crew just now. Reload the page to try again.</p>
		{:else}
			{#if busy.length > 0}
				<section aria-labelledby="busy-title">
					<h2 id="busy-title" class="section-label">Busy for you right now · {busy.length}</h2>
					<ul class="grid">
						{#each busy as member (member.agent_type)}{@render card(member)}{/each}
					</ul>
				</section>
			{/if}
			<section aria-labelledby="ready-title">
				<h2 id="ready-title" class="section-label">Ready when you are · {ready.length}</h2>
				<ul class="grid">
					{#each ready as member (member.agent_type)}{@render card(member)}{/each}
				</ul>
			</section>
		{/if}
	</div>
</div>

<style>
	.crew-page {
		height: 100%;
		overflow-y: auto;
		padding: 32px clamp(16px, 4vw, 56px) 56px;
		box-sizing: border-box;
	}
	.crew-page__inner {
		display: flex;
		flex-direction: column;
		gap: 28px;
		max-width: 1080px;
		margin: 0 auto;
	}
	.crew-page__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.crew-page__lede {
		max-width: 60ch;
		margin: 8px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.crew-page__empty {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.section-label {
		margin: 0 8px 12px;
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		font-weight: 400;
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(min(100%, 300px), 1fr));
		gap: 12px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.card {
		display: flex;
		align-items: flex-start;
		gap: 16px;
		padding: 20px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.card[data-state='working'],
	.card[data-state='waiting'] {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.card__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
	}
	.card__top {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
	}
	.card__name {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.25rem;
		font-weight: 700;
	}
	.card__custom,
	.card__badge {
		padding: 2px 10px;
		border-radius: var(--md-sys-shape-corner-full);
		font-size: 0.75rem;
		font-weight: 650;
	}
	.card__custom {
		border: 1px solid var(--md-sys-color-outline-variant);
		color: var(--md-sys-color-on-surface-variant);
	}
	.card__badge {
		margin-left: auto;
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
	.card__badge[data-state='waiting'] {
		background: var(--md-sys-color-tertiary);
		color: var(--md-sys-color-on-tertiary);
	}
	.card__badge[data-state='done'] {
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface);
	}
	.card__role {
		margin: 0;
		color: inherit;
		font-size: 0.875rem;
		line-height: 1.45;
		opacity: 0.8;
	}
	.card__status {
		margin: 2px 0 0;
		font-size: 0.875rem;
		font-weight: 600;
		line-height: 1.45;
	}
</style>
