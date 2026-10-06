<script lang="ts">
	// An agent's thinking, collapsed to one quiet line by default; opens in place.

	let {
		steps,
		live = false,
	}: {
		/** Thinking steps, oldest first. */
		steps: string[];
		/** No reply yet: the agent may still be thinking. */
		live?: boolean;
	} = $props();

	let open = $state(false);
	const uid = $props.id();
</script>

<div class="reasoning" class:reasoning--open={open}>
	<button
		type="button"
		class="reasoning__toggle"
		aria-expanded={open}
		aria-controls="{uid}-body"
		onclick={() => (open = !open)}
	>
		<svg class="reasoning__icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
			<path d="M9 18h6M10 21h4M12 3a6 6 0 0 0-3.5 10.9c.6.4 1 1.1 1 1.8V16h5v-.3c0-.7.4-1.4 1-1.8A6 6 0 0 0 12 3Z" />
		</svg>
		<span class="reasoning__label">{live ? 'Thinking' : 'Thought process'}</span>
		{#if steps.length > 1}
			<span class="nomi-meta reasoning__count">{steps.length} steps</span>
		{/if}
		<svg class="reasoning__chevron" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m6 9 6 6 6-6" /></svg>
	</button>
	<div id="{uid}-body" class="reasoning__body" hidden={!open}>
		{#each steps as step, i (i)}
			<p class="reasoning__step">{step}</p>
		{/each}
	</div>
</div>

<style>
	.reasoning {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 6px;
		max-width: 100%;
	}
	.reasoning__toggle {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		min-height: 36px;
		padding: 0 12px 0 10px;
		border: none;
		border-radius: 18px;
		background: var(--md-sys-color-surface-container);
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-size: 0.875rem;
		font-weight: 600;
		cursor: pointer;
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-fast);
	}
	.reasoning__toggle:hover {
		background: var(--md-sys-color-surface-container-high);
	}
	.reasoning--open .reasoning__toggle {
		border-radius: var(--md-sys-shape-corner-medium);
		color: var(--md-sys-color-on-surface);
	}
	.reasoning__toggle:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.reasoning__count {
		text-transform: none;
	}
	.reasoning__chevron {
		transition: rotate var(--nomi-motion-spatial-fast);
	}
	.reasoning--open .reasoning__chevron {
		rotate: 180deg;
	}
	.reasoning__body {
		display: flex;
		flex-direction: column;
		gap: 10px;
		max-width: 68ch;
		max-height: 360px;
		overflow-y: auto;
		padding: 14px 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface-variant);
	}
	.reasoning__body[hidden] {
		display: none;
	}
	.reasoning__step {
		margin: 0;
		font-size: 0.875rem;
		line-height: 1.55;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
	@media (prefers-reduced-motion: reduce) {
		.reasoning__toggle,
		.reasoning__chevron {
			transition: none;
		}
	}
</style>
