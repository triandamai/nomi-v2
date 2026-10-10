<!-- Everything in Nomi in one place, opened from the drawer's More. Each place can be pinned to
     the drawer (or unpinned) right here; the drawer's order is set in Preferences. -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Snackbar from '$lib/components/m3/Snackbar.svelte';
	import IconPin from '$lib/components/icons/IconPin.svelte';
	import IconLock from '$lib/components/icons/IconLock.svelte';
	import { FEATURES, MORE_SECTIONS, PERMANENT, isPinnable, type FeatureId } from '$lib/features';
	import { currentPins, savePins } from '$lib/drawerStore.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const pins = $derived(currentPins(data.preferences.drawer_pins));
	let toast = $state('');
	let toastOpen = $state(false);

	async function togglePin(id: FeatureId) {
		if (!isPinnable(id)) return;
		const next = pins.includes(id) ? pins.filter((p) => p !== id) : [...pins, id];
		const ok = await savePins(next);
		toast = ok ? m.drawer_saved() : m.drawer_save_failed();
		toastOpen = true;
	}
</script>

<div class="more">
	<div class="more__inner">
		<header>
			<h1 class="md-display-small more__title">{m.more_title()}</h1>
			<p class="md-body-large more__lede">{m.more_lede()}</p>
			<a class="more__customize" href="/preferences#drawer">{m.more_customize()}</a>
		</header>

		{#each MORE_SECTIONS as section (section.ids[0])}
			<section class="more__section">
				<h2 class="more__section-title">{section.title()}</h2>
				<ul class="more__grid">
					{#each section.ids as id (id)}
						{@const feature = FEATURES[id]}
						{@const pinned = isPinnable(id) && pins.includes(id)}
						<li class="tile" class:tile--pinned={pinned || PERMANENT.includes(id)}>
							<a class="tile__link" href={feature.href}>
								<span class="tile__icon"><feature.icon size={24} /></span>
								<span class="tile__text">
									<span class="tile__name">{feature.label()}</span>
									<span class="tile__desc">{feature.description()}</span>
								</span>
							</a>
							{#if PERMANENT.includes(id)}
								<span class="tile__lock" title={m.drawer_always()}><IconLock size={16} /><span class="sr-only">{m.drawer_always()}</span></span>
							{:else if isPinnable(id)}
								<button
									type="button"
									class="tile__pin"
									aria-pressed={pinned}
									aria-label={pinned ? m.more_unpin({ name: feature.label() }) : m.more_pin({ name: feature.label() })}
									title={pinned ? m.more_unpin({ name: feature.label() }) : m.more_pin({ name: feature.label() })}
									onclick={() => togglePin(id)}
								>
									<IconPin size={18} />
								</button>
							{/if}
						</li>
					{/each}
				</ul>
			</section>
		{/each}
	</div>
</div>

<Snackbar bind:open={toastOpen} message={toast} />

<style>
	.more {
		height: 100%;
		overflow-y: auto;
		padding: 32px clamp(16px, 4vw, 56px) 56px;
		box-sizing: border-box;
	}
	.more__inner {
		display: flex;
		flex-direction: column;
		gap: 28px;
		max-width: 1080px;
		margin: 0 auto;
	}
	.more__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.more__lede {
		max-width: 60ch;
		margin: 8px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.more__customize {
		display: inline-flex;
		align-items: center;
		min-height: 40px;
		margin-top: 8px;
		color: var(--md-sys-color-primary);
		font-weight: 600;
		text-underline-offset: 3px;
	}
	.more__section-title {
		margin: 0 0 12px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		font-weight: 600;
	}
	.more__grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
		gap: 12px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.tile {
		position: relative;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container);
		transition: background-color var(--nomi-motion-effects-fast);
	}
	.tile:hover {
		background: var(--md-sys-color-surface-container-high);
	}
	.tile__link {
		display: flex;
		align-items: center;
		gap: 16px;
		min-height: 88px;
		padding: 16px 56px 16px 16px;
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
		border-radius: inherit;
	}
	.tile__link:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.tile__icon {
		display: grid;
		place-items: center;
		width: 48px;
		height: 48px;
		flex: none;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
	}
	.tile--pinned .tile__icon {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.tile__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.tile__name {
		font-weight: 700;
	}
	.tile__desc {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		line-height: 1.35;
	}
	.tile__pin,
	.tile__lock {
		position: absolute;
		top: 8px;
		right: 8px;
		display: grid;
		place-items: center;
		width: 40px;
		height: 40px;
		border-radius: var(--md-sys-shape-corner-full);
		color: var(--md-sys-color-on-surface-variant);
	}
	.tile__lock {
		opacity: 0.6;
	}
	.tile__pin {
		border: none;
		background: none;
		cursor: pointer;
	}
	.tile__pin:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.tile__pin:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
	}
	.tile__pin[aria-pressed='true'] {
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
</style>
