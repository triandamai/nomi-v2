<script lang="ts">
	import GlassBackdrop from '$lib/components/GlassBackdrop.svelte';
	import HomeStatusCards from '$lib/components/HomeStatusCards.svelte';
	import HomeComposer from './blocks/HomeComposer.svelte';
	import HomeCrew from './blocks/HomeCrew.svelte';
	import HomeGreeting from './blocks/HomeGreeting.svelte';
	import HomeRecent from './blocks/HomeRecent.svelte';
	import type { HomeData } from './blocks/homeData';
	import type { Snippet } from 'svelte';
	import type { HomeBlock, HomeLayout } from '$lib/homeLayouts';

	// Renders a home layout from $lib/homeLayouts: its backdrop, then each row's blocks.

	let {
		layout,
		home,
		error = null,
		notice,
	}: {
		layout: HomeLayout;
		home: HomeData;
		/** The composer's last send error. */
		error?: string | null;
		/** Anything the page shows above the layout (e.g. a redirect's message). */
		notice?: Snippet;
	} = $props();

	// The glass stage, unless the person turned the animated background off (Preferences).
	const glass = $derived(layout.backdrop === 'glass' && home.preferences.animated_backdrop !== false);
	// A hero greeting already shows Nomi's mark; the stage then holds the rest of the crew.
	const hasHeroMark = $derived(layout.rows.some((row) => row.columns.flat().some((item) => item.block === 'greeting' && item.size === 'hero')));
	const crewBusy = $derived(home.crew.some((member) => member.state === 'working'));
</script>

{#snippet block(item: HomeBlock, center: boolean)}
	{#if item.block === 'greeting'}
		<HomeGreeting {home} size={item.size} />
	{:else if item.block === 'composer'}
		<HomeComposer suggestions={item.suggestions} {error} {glass} {center} />
	{:else if item.block === 'crew'}
		<HomeCrew crew={home.crew} style={item.style} limit={item.limit} />
	{:else if item.block === 'status'}
		{#if home.summary}<HomeStatusCards summary={home.summary} />{/if}
	{:else if item.block === 'recent'}
		<HomeRecent sessions={home.recentSessions} style={item.style} limit={item.limit} />
	{/if}
{/snippet}

<div class="home-stage" data-layout={layout.id}>
	{#if glass}<GlassBackdrop busy={crewBusy} withoutNomi={hasHeroMark} />{/if}
	<div class="home">
		<div class="home__inner">
			{@render notice?.()}
			{#each layout.rows as row, r (r)}
				<div
					class="home__row"
					class:home__row--narrow={row.width === 'narrow'}
					class:home__row--center={row.align === 'center'}
					class:home__row--split={row.columns.length > 1}
					style:--columns={row.ratio ? row.ratio.map((fr) => `minmax(0, ${fr}fr)`).join(' ') : `repeat(${row.columns.length}, minmax(0, 1fr))`}
				>
					{#each row.columns as column, c (c)}
						<div class="home__column">
							{#each column as item, i (i)}
								{@render block(item, row.align === 'center')}
							{/each}
						</div>
					{/each}
				</div>
			{/each}
		</div>
	</div>
</div>

<style>
	.home-stage {
		position: relative;
		isolation: isolate;
		height: 100%;
	}
	.home {
		position: relative;
		z-index: 1;
		height: 100%;
		overflow-y: auto;
		padding: 24px clamp(14px, 4vw, 44px) 44px;
		box-sizing: border-box;
	}
	.home__inner {
		max-width: 1180px;
		margin: 0 auto;
		display: flex;
		flex-direction: column;
		gap: 36px;
	}
	.home__row {
		min-width: 0;
	}
	.home__row--narrow {
		width: 100%;
		max-width: 760px;
		margin: 0 auto;
	}
	.home__row--center .home__column {
		align-items: center;
		text-align: center;
	}
	/* Side-by-side columns wrap to one under the other until there's room. */
	.home__row--split {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(100%, 380px), 1fr));
		gap: 22px;
		align-items: start;
	}
	@media (min-width: 1100px) {
		.home__row--split {
			grid-template-columns: var(--columns);
		}
	}
	.home__column {
		display: flex;
		flex-direction: column;
		gap: 20px;
		min-width: 0;
	}
</style>
