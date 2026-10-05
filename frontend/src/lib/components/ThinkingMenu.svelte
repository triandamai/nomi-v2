<script lang="ts">
	import Menu from '$lib/components/m3/Menu.svelte';
	import MenuItem from '$lib/components/m3/MenuItem.svelte';
	import type { ThinkingLevel } from '$lib/types';

	// The composer's thinking-level picker: how much the model reasons before it answers in this
	// chat. Saved per chat (PUT /api/sessions/:id/thinking, via the page's setThinking action).

	let {
		level,
		onchange,
	}: {
		level: ThinkingLevel;
		onchange: (level: ThinkingLevel) => void;
	} = $props();

	const LEVELS: { value: ThinkingLevel; label: string; hint: string }[] = [
		{ value: 'off', label: 'Off', hint: 'Fastest. Answers straight away' },
		{ value: 'low', label: 'Low', hint: 'A quick think first' },
		{ value: 'medium', label: 'Medium', hint: 'Balanced (the default)' },
		{ value: 'high', label: 'High', hint: 'Slower, for tricky problems' },
	];

	let open = $state(false);
	const current = $derived(LEVELS.find((l) => l.value === level) ?? LEVELS[2]);
</script>

<Menu bind:open>
	{#snippet trigger({ toggle })}
		<button type="button" class="thinking" class:thinking--off={level === 'off'} onclick={toggle} aria-label="Thinking: {current.label}" title="Thinking level">
			<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
				<path d="M9 18h6M10 21h4M12 3a6 6 0 0 0-3.5 10.9c.6.4 1 1.1 1 1.8V16h5v-.3c0-.7.4-1.4 1-1.8A6 6 0 0 0 12 3Z" />
			</svg>
			<span class="thinking__label">{current.label}</span>
		</button>
	{/snippet}
	<div class="levels">
		<p class="md-label-medium levels__title">Thinking</p>
		{#each LEVELS as option (option.value)}
			<MenuItem
				type="button"
				selected={option.value === level}
				onclick={() => {
					open = false;
					if (option.value !== level) onchange(option.value);
				}}
			>
				<span class="levels__item">
					<span class="levels__label">{option.label}</span>
					<span class="levels__hint">{option.hint}</span>
				</span>
			</MenuItem>
		{/each}
	</div>
</Menu>

<style>
	.thinking {
		flex: none;
		display: inline-flex;
		align-items: center;
		gap: 6px;
		height: 40px;
		padding: 0 12px 0 10px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font: inherit;
		font-size: 0.8125rem;
		font-weight: 650;
		cursor: pointer;
		transition: border-radius var(--nomi-motion-spatial-fast);
	}
	.thinking:hover {
		border-radius: var(--md-sys-shape-corner-medium);
	}
	.thinking:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.thinking--off {
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
	}
	/* A narrow chat (the project page's side column) keeps just the bulb. */
	@container (max-width: 520px) {
		.thinking__label {
			display: none;
		}
		.thinking {
			padding: 0 10px;
		}
	}
	.levels {
		width: 240px;
	}
	.levels__title {
		margin: 0;
		padding: 4px 8px 8px;
		color: var(--md-sys-color-on-surface-variant);
	}
	.levels__item {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		text-align: left;
	}
	.levels__label {
		font-weight: 650;
	}
	.levels__hint {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
