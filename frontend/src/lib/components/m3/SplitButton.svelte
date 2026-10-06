<script lang="ts">
	import type { Snippet } from 'svelte';
	import Menu from './Menu.svelte';

	// M3 Expressive split button: a primary action and a menu trigger joined with tight inner
	// corners. The trigger half turns fully round while its menu is open.

	let {
		variant = 'tonal',
		menuLabel,
		onclick,
		children,
		menu,
		menuOpen = $bindable(false),
	}: {
		variant?: 'filled' | 'tonal';
		/** aria-label for the chevron half, e.g. "Choose agent". */
		menuLabel: string;
		onclick?: () => void;
		children: Snippet;
		menu: Snippet;
		menuOpen?: boolean;
	} = $props();
</script>

<div class="m3-split m3-split--{variant}">
	<button type="button" class="m3-split__main" {onclick}>{@render children()}</button>
	<Menu bind:open={menuOpen}>
		{#snippet trigger({ toggle })}
			<button
				type="button"
				class="m3-split__trigger"
				class:m3-split__trigger--open={menuOpen}
				aria-label={menuLabel}
				aria-haspopup="menu"
				aria-expanded={menuOpen}
				onclick={toggle}
			>
				<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m6 9 6 6 6-6" /></svg>
			</button>
		{/snippet}
		{@render menu()}
	</Menu>
</div>

<style>
	.m3-split {
		display: inline-flex;
		gap: 2px;
	}
	.m3-split__main,
	.m3-split__trigger {
		height: 44px;
		border: none;
		cursor: pointer;
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: var(--md-sys-typescale-label-large-size);
		font-weight: 600;
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-fast);
	}
	.m3-split__main {
		padding: 0 14px 0 16px;
		border-radius: 22px 6px 6px 22px;
	}
	.m3-split__trigger {
		width: 40px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: 6px 22px 22px 6px;
	}
	.m3-split__trigger svg {
		transition: rotate var(--nomi-motion-spatial-fast);
	}
	.m3-split__trigger--open {
		border-radius: 22px;
	}
	.m3-split__trigger--open svg {
		rotate: 180deg;
	}
	.m3-split--tonal .m3-split__main,
	.m3-split--tonal .m3-split__trigger {
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface);
	}
	.m3-split--filled .m3-split__main,
	.m3-split--filled .m3-split__trigger {
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
	.m3-split__main:hover,
	.m3-split__trigger:hover {
		filter: brightness(0.96);
	}
</style>
