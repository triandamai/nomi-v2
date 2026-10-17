<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import { crewPreview, rosterKey, type CrewRosterMember } from '$lib/crew';

	// The crew, busiest first, with what each is doing right now. `card` is a dark stage card
	// with a list; `strip` is one scrolling row of members, for layouts that put the crew under
	// the composer. Either way the Crew page has everyone.

	let { crew, style = 'card', limit = 4 }: { crew: CrewRosterMember[]; style?: 'card' | 'strip'; limit?: number } = $props();

	const workingCount = $derived(crew.filter((member) => member.state === 'working').length);
	const preview = $derived(crewPreview(crew, limit));
</script>

{#snippet seeAll()}
	<a class="crew__all" href="/crew">
		{m.home_see_all({ count: crew.length })}
		<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M5 12h14M13 6l6 6-6 6" /></svg>
	</a>
{/snippet}

{#snippet badge(member: CrewRosterMember)}
	{#if member.state === 'waiting'}
		<span class="crew__badge">{m.home_needs_you()}</span>
	{:else if member.state === 'done'}
		<span class="crew__badge crew__badge--done">{m.common_done()}</span>
	{/if}
{/snippet}

<section class="crew crew--{style}" aria-labelledby="crew-heading">
	<div class="crew__head">
		<h2 id="crew-heading" class="crew__title">{m.home_your_crew()}</h2>
		{#if workingCount > 0}
			<span class="nomi-meta crew__count">{m.home_working_count({ count: workingCount })}</span>
		{/if}
		{#if style === 'strip' && preview.more > 0}{@render seeAll()}{/if}
	</div>
	<ul class="crew__list">
		{#each preview.shown as member (member.agent_type)}
			{@const key = rosterKey(member.agent_type)}
			<li class="crew__item" data-state={member.state}>
				<AgentShape agent={key} size={style === 'strip' ? 40 : 44} face={key === 'nomi'} working={member.state === 'working'} />
				<span class="crew__text">
					<span class="crew__name">{member.name}</span>
					<span class="crew__role" title={member.role}>
						{#if member.state === 'idle'}{member.role}{:else}{member.status}{/if}
					</span>
				</span>
				{@render badge(member)}
			</li>
		{/each}
	</ul>
	{#if style === 'card' && preview.more > 0}{@render seeAll()}{/if}
</section>

<style>
	.crew {
		min-width: 0;
	}
	.crew--card {
		padding: 22px;
		border-radius: 40px 40px 40px 12px;
		background: var(--nomi-color-stage);
		color: var(--nomi-color-on-stage);
		--crew-ink: var(--nomi-color-on-stage);
	}
	.crew--strip {
		color: var(--md-sys-color-on-surface);
		--crew-ink: var(--md-sys-color-on-surface);
	}
	.crew__head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
		margin-bottom: 12px;
	}
	.crew--strip .crew__head {
		align-items: center;
		justify-content: flex-start;
	}
	.crew--strip .crew__all {
		margin: 0 0 0 auto;
	}
	.crew__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.5rem;
		font-weight: 700;
		letter-spacing: -0.02em;
	}
	.crew--strip .crew__title {
		font-size: 1.25rem;
	}
	.crew__count {
		color: var(--crew-ink);
		opacity: 0.8;
	}
	.crew__list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.crew__item {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 0;
	}
	/* The strip: one row of members that scrolls sideways on narrow screens. */
	.crew--strip .crew__list {
		flex-direction: row;
		gap: 10px;
		overflow-x: auto;
		scroll-snap-type: x proximity;
		scrollbar-width: none;
		margin: 0 -4px;
		padding: 4px;
	}
	.crew--strip .crew__list::-webkit-scrollbar {
		display: none;
	}
	.crew--strip .crew__item {
		flex: none;
		scroll-snap-align: start;
		width: 196px;
		box-sizing: border-box;
		padding: 10px 14px 10px 10px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: color-mix(in srgb, var(--md-sys-color-surface-container-lowest) 72%, transparent);
		box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--md-sys-color-surface-container-lowest) 60%, transparent);
	}
	.crew__text {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}
	.crew__name {
		font-weight: 650;
		font-size: 0.9375rem;
	}
	.crew__role {
		font-size: 0.875rem;
		opacity: 0.8;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.crew__item[data-state='idle'] .crew__role {
		opacity: 0.6;
	}
	.crew__badge {
		flex: none;
		margin-left: auto;
		padding: 4px 10px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--nomi-gradient-ember);
		color: var(--nomi-on-gradient-ember);
		font-size: 0.75rem;
		font-weight: 700;
	}
	.crew__badge--done {
		background: color-mix(in srgb, var(--crew-ink) 14%, transparent);
		color: var(--crew-ink);
	}
	.crew--strip .crew__item:has(.crew__badge) {
		width: 248px;
	}
	.crew__all {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		min-height: 40px;
		margin-top: 12px;
		padding: 0 14px;
		border-radius: 20px;
		background: color-mix(in srgb, var(--crew-ink) 12%, transparent);
		color: var(--crew-ink);
		font-size: 0.875rem;
		font-weight: 650;
		text-decoration: none;
		transition:
			background-color var(--nomi-motion-effects-fast),
			border-radius var(--nomi-motion-spatial-fast);
	}
	.crew__all:hover {
		background: color-mix(in srgb, var(--crew-ink) 20%, transparent);
	}
	.crew__all:active {
		border-radius: var(--md-sys-shape-corner-medium);
	}
	.crew__all:focus-visible {
		outline: 2px solid var(--crew-ink);
		outline-offset: 2px;
	}
	@media (pointer: coarse) {
		.crew__all {
			min-height: 44px;
		}
	}
</style>
