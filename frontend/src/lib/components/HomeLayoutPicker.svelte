<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import { HOME_LAYOUTS, homeLayout, type HomeBlockKind } from '$lib/homeLayouts';

	// Preferences → Home: which home page to open on. Each option's thumbnail is drawn from the
	// layout's own config, so a new layout gets one without any work here.

	let { current }: { current: string | null | undefined } = $props();

	let picked = $state('');
	$effect.pre(() => {
		picked = homeLayout(current).id;
	});

	// Thumbnail height of each kind of block, in px.
	const BLOCK_HEIGHT: Record<HomeBlockKind, number> = { greeting: 9, composer: 14, crew: 12, status: 12, recent: 10 };

	const keep = () => async ({ update }: { update: (opts?: { reset?: boolean }) => Promise<void> }) => {
		await update({ reset: false });
	};
</script>

<form method="POST" action="/preferences?/updateHomeLayout" use:enhance={keep} class="layouts">
	{#each HOME_LAYOUTS as layout (layout.id)}
		<button
			type="submit"
			name="home_layout"
			value={layout.id}
			class="layout"
			class:layout--selected={picked === layout.id}
			aria-pressed={picked === layout.id}
			onclick={() => (picked = layout.id)}
		>
			<span class="layout__preview" class:layout__preview--glass={layout.backdrop === 'glass'} aria-hidden="true">
				{#each layout.rows as row, r (r)}
					<span class="layout__row" class:layout__row--narrow={row.width === 'narrow'}>
						{#each row.columns as column, c (c)}
							<span class="layout__column" style:flex={row.ratio?.[c] ?? 1}>
								{#each column as item, i (i)}
									<span class="layout__block layout__block--{item.block}" style:height="{BLOCK_HEIGHT[item.block]}px"></span>
								{/each}
							</span>
						{/each}
					</span>
				{/each}
			</span>
			<span class="layout__name">{layout.name()}</span>
			<span class="layout__hint">{layout.description()}</span>
		</button>
	{/each}
</form>

<style>
	.layouts {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(min(100%, 220px), 1fr));
		gap: 12px;
	}
	.layout {
		display: flex;
		flex-direction: column;
		align-items: stretch;
		gap: 4px;
		padding: 10px 10px 14px;
		border: none;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		text-align: start;
		cursor: pointer;
		box-shadow: inset 0 0 0 1px var(--md-sys-color-outline-variant);
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			box-shadow var(--nomi-motion-effects-fast),
			background-color var(--nomi-motion-effects-fast);
	}
	.layout:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	.layout:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.layout--selected {
		border-radius: var(--md-sys-shape-corner-large);
		box-shadow: inset 0 0 0 2px var(--md-sys-color-primary);
	}
	.layout__preview {
		display: flex;
		flex-direction: column;
		gap: 5px;
		height: 112px;
		margin-bottom: 8px;
		padding: 10px;
		box-sizing: border-box;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--nomi-color-ground);
	}
	.layout__preview--glass {
		background:
			radial-gradient(circle at 18% 12%, color-mix(in srgb, var(--nomi-action-1) 70%, transparent), transparent 55%),
			radial-gradient(circle at 88% 40%, color-mix(in srgb, var(--nomi-action-2) 60%, transparent), transparent 55%),
			radial-gradient(circle at 40% 100%, color-mix(in srgb, var(--nomi-action-3) 55%, transparent), transparent 60%),
			var(--nomi-color-ground);
	}
	.layout__row {
		display: flex;
		gap: 5px;
		flex: none;
	}
	.layout__row--narrow {
		padding: 0 18%;
	}
	.layout__column {
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
	}
	.layout__block {
		display: block;
		border-radius: 4px;
		background: var(--md-sys-color-surface-container-lowest);
	}
	.layout__block--greeting {
		width: 62%;
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 22%, transparent);
	}
	.layout__row--narrow .layout__block--greeting {
		align-self: center;
	}
	.layout__block--crew {
		background: var(--nomi-color-stage);
	}
	.layout__preview--glass .layout__block--crew {
		background: color-mix(in srgb, var(--md-sys-color-surface-container-lowest) 72%, transparent);
	}
	.layout__name {
		font-weight: 650;
		font-size: 0.9375rem;
	}
	.layout__hint {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
