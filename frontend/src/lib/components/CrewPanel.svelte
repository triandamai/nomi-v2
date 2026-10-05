<script lang="ts">
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import type { CrewMember } from '$lib/crew';

	// The chat's crew: every agent, with what it's doing right now. Members working in this chat
	// come first and spin; the rest show their role and stand by.

	let { members }: { members: CrewMember[] } = $props();

	const sorted = $derived(
		[...members].sort((a, b) => Number(b.working) - Number(a.working) || Number(b.involved) - Number(a.involved)),
	);
	const workingCount = $derived(members.filter((m) => m.working).length);
</script>

<section class="crew" aria-labelledby="crew-panel-title">
	<div class="crew__head">
		<h2 id="crew-panel-title" class="crew__title">Your crew</h2>
		<span class="crew__count" class:crew__count--live={workingCount > 0}>
			{workingCount > 0 ? `${workingCount} working` : 'All idle'}
		</span>
	</div>
	<ul class="crew__list">
		{#each sorted as member (member.key)}
			<li class="crew__item" class:crew__item--working={member.working} class:crew__item--idle={!member.involved}>
				<AgentShape agent={member.key} size={40} working={member.working} face={member.key === 'nomi'} />
				<span class="crew__text">
					<span class="crew__name">
						{member.name}
						{#if member.working}<span class="crew__live">Live</span>{/if}
					</span>
					<span class="crew__status">{member.involved ? member.status : member.role}</span>
				</span>
			</li>
		{/each}
	</ul>
</section>

<style>
	.crew {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 22px 16px 16px;
		border-radius: 32px 32px 32px 10px;
		background: var(--nomi-color-stage);
		color: var(--nomi-color-on-stage);
	}
	.crew__head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		padding: 0 8px;
	}
	.crew__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.5rem;
		font-weight: 700;
		letter-spacing: -0.02em;
	}
	.crew__count {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		opacity: 0.7;
	}
	.crew__count--live {
		color: #7ce0a3;
		opacity: 1;
	}
	.crew__list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.crew__item {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 8px;
		border-radius: 20px;
		transition: background-color var(--nomi-motion-effects-default);
	}
	.crew__item--working {
		background: color-mix(in srgb, var(--nomi-color-on-stage) 9%, transparent);
	}
	.crew__item--idle {
		opacity: 0.62;
	}
	.crew__text {
		display: flex;
		flex-direction: column;
		min-width: 0;
		gap: 1px;
	}
	.crew__name {
		display: flex;
		align-items: center;
		gap: 8px;
		font-weight: 650;
		font-size: 0.9375rem;
	}
	.crew__live {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.625rem;
		font-weight: 500;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: #7ce0a3;
	}
	.crew__status {
		font-size: 0.8125rem;
		line-height: 1.35;
		opacity: 0.78;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}
</style>
