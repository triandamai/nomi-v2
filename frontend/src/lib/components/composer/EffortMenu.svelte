<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Menu from '$lib/components/m3/Menu.svelte';
	import IconCheck from '$lib/components/icons/IconCheck.svelte';
	import IconChevronDown from '$lib/components/icons/IconChevronDown.svelte';
	import type { ThinkingLevel } from '$lib/types';

	// The composer's thinking pill: how much the model reasons before it answers in this chat.
	// Saved per chat (PUT /api/sessions/:id/thinking, via the page's setThinking action).

	let {
		level,
		onchange,
		disabled = false,
	}: {
		level: ThinkingLevel;
		onchange: (level: ThinkingLevel) => void;
		disabled?: boolean;
	} = $props();

	// "Default" is the backend's medium, so it reads as the setting most people leave alone.
	const OPTIONS: { value: ThinkingLevel; label: string }[] = [
		{ value: 'medium', label: m.composer_effort_default() },
		{ value: 'low', label: m.think_low() },
		{ value: 'high', label: m.think_high() },
		{ value: 'max', label: m.composer_effort_max() },
	];

	let open = $state(false);
	const current = $derived(OPTIONS.find((option) => option.value === level) ?? OPTIONS[0]);
</script>

<Menu bind:open align="start">
	{#snippet trigger({ toggle })}
		<button
			type="button"
			class="effort"
			aria-haspopup="menu"
			aria-expanded={open}
			aria-label={m.think_label({ level: current.label })}
			title={m.think_title()}
			{disabled}
			onclick={toggle}
		>
			<svg class="effort__bulb" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
				<path d="M9 18h6M10 21h4M12 3a6 6 0 0 0-3.5 10.9c.6.4 1 1.1 1 1.8V16h5v-.3c0-.7.4-1.4 1-1.8A6 6 0 0 0 12 3Z" />
			</svg>
			<span class="effort__label">{current.label}</span>
			<IconChevronDown size={16} />
		</button>
	{/snippet}
	<div class="levels">
		<p class="levels__title">{m.think_heading()}</p>
		{#each OPTIONS as option (option.value)}
			{@const active = option.value === level}
			<button
				type="button"
				role="menuitemradio"
				tabindex="-1"
				aria-checked={active}
				class="level"
				class:level--active={active}
				onclick={() => {
					open = false;
					if (!active) onchange(option.value);
				}}
			>
				<span>{option.label}</span>
				{#if active}<span class="level__check" data-check><IconCheck size={18} /></span>{/if}
			</button>
		{/each}
	</div>
</Menu>

<style>
	.effort {
		flex: none;
		display: inline-flex;
		align-items: center;
		gap: 4px;
		height: 40px;
		padding: 0 8px 0 10px;
		border: none;
		border-radius: 20px;
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-size: 0.8125rem;
		font-weight: 650;
		white-space: nowrap;
		cursor: pointer;
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-fast);
	}
	.effort:hover:not(:disabled) {
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface);
	}
	.effort:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.effort:disabled {
		opacity: 0.38;
		cursor: not-allowed;
	}
	@media (pointer: coarse) {
		.effort {
			height: 44px;
			border-radius: 22px;
		}
	}
	/* A narrow composer keeps the label (it's the information) and drops the bulb. */
	@container (max-width: 480px) {
		.effort__bulb {
			display: none;
		}
	}
	.levels {
		width: 200px;
	}
	.levels__title {
		margin: 0;
		padding: 4px 10px 6px;
		font-size: 0.75rem;
		font-weight: 650;
		color: var(--md-sys-color-on-surface-variant);
	}
	.level {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		width: 100%;
		min-height: 40px;
		padding: 0 10px 0 12px;
		border: none;
		border-radius: var(--md-sys-shape-corner-medium);
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: 0.875rem;
		text-align: left;
		cursor: pointer;
		transition: background-color var(--nomi-motion-effects-fast);
	}
	.level:hover,
	.level:focus-visible {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
		outline: none;
	}
	.level--active {
		color: var(--md-sys-color-primary);
		font-weight: 650;
	}
	.level__check {
		display: inline-flex;
		flex: none;
	}
	@media (pointer: coarse) {
		.level {
			min-height: 44px;
		}
	}
</style>
