<!-- Options side by side: a grid where there's room (criteria down the side, options across), one
     card per option on phones. The recommended option is marked, not coloured loudly. -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import ViewFrame from './ViewFrame.svelte';
	import Value from './Value.svelte';
	import type { ComparisonView, ViewProps } from '$lib/views/types';

	let { view }: ViewProps<ComparisonView> = $props();
</script>

<ViewFrame title={view.title}>
	<div class="compare" style:--options={view.options.length}>
		<div class="compare__grid" role="table">
			<div class="compare__row compare__row--head" role="row">
				<span role="columnheader"><span class="sr-only">{m.view_criterion()}</span></span>
				{#each view.options as option, i (i)}
					<span class="compare__option" class:compare__option--best={view.recommended === i} role="columnheader">
						{option}
						{#if view.recommended === i}<span class="compare__badge">{m.view_recommended()}</span>{/if}
					</span>
				{/each}
			</div>
			{#each view.rows as row, r (r)}
				<div class="compare__row" role="row">
					<span class="compare__label" role="rowheader">{row.label}</span>
					{#each row.values as value, i (i)}
						<span class="compare__cell" class:compare__cell--best={view.recommended === i} role="cell"><Value {value} /></span>
					{/each}
				</div>
			{/each}
		</div>

		<div class="compare__cards">
			{#each view.options as option, i (i)}
				<div class="compare__card" class:compare__card--best={view.recommended === i}>
					<span class="compare__card-title">
						{option}
						{#if view.recommended === i}<span class="compare__badge">{m.view_recommended()}</span>{/if}
					</span>
					<dl class="compare__facts">
						{#each view.rows as row, r (r)}
							<div class="compare__fact">
								<dt>{row.label}</dt>
								<dd><Value value={row.values[i]} /></dd>
							</div>
						{/each}
					</dl>
				</div>
			{/each}
		</div>
	</div>
	{#if view.verdict}<p class="compare__verdict">{view.verdict}</p>{/if}
</ViewFrame>

<style>
	.compare {
		container-type: inline-size;
	}
	.compare__grid {
		display: grid;
		grid-template-columns: minmax(96px, 0.8fr) repeat(var(--options), minmax(0, 1fr));
	}
	.compare__row {
		display: contents;
	}
	.compare__row > * {
		padding: 9px 10px;
		border-top: 1px solid var(--md-sys-color-outline-variant);
		font-size: 0.9375rem;
		line-height: 1.35;
		overflow-wrap: anywhere;
	}
	.compare__row--head > * {
		border-top: none;
		padding-top: 0;
	}
	.compare__label {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.compare__option {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 4px;
		font-weight: 700;
	}
	.compare__cell--best,
	.compare__option--best {
		background: var(--md-sys-color-surface-container-low);
	}
	.compare__option--best {
		border-radius: var(--md-sys-shape-corner-medium) var(--md-sys-shape-corner-medium) 0 0;
		padding-top: 9px !important;
	}
	.compare__badge {
		padding: 2px 8px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
		font-size: 0.75rem;
		font-weight: 600;
	}
	.compare__cards {
		display: none;
		flex-direction: column;
		gap: 10px;
	}
	.compare__card {
		padding: 12px 14px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-low);
	}
	.compare__card--best {
		box-shadow: inset 0 0 0 2px var(--md-sys-color-primary);
	}
	.compare__card-title {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
		font-weight: 700;
	}
	.compare__facts {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin: 10px 0 0;
	}
	.compare__fact {
		display: flex;
		justify-content: space-between;
		gap: 12px;
		font-size: 0.9375rem;
	}
	.compare__fact dt {
		color: var(--md-sys-color-on-surface-variant);
	}
	.compare__fact dd {
		margin: 0;
		text-align: end;
		font-weight: 600;
	}
	@container (max-width: 460px) {
		.compare__grid {
			display: none;
		}
		.compare__cards {
			display: flex;
		}
	}
	.compare__verdict {
		margin: 0;
		line-height: 1.45;
	}
</style>
