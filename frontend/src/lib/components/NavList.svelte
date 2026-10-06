<script lang="ts">
	import type { Component } from 'svelte';

	// M3 navigation drawer items: an icon in its active-indicator pill, then the label. As a rail
	// (a collapsed sidebar) the label sits under the pill. Shared by the app and admin sidebars.
	let {
		items,
		rail = false,
		isActive,
		label,
		onnavigate,
	}: {
		items: { href: string; label: string; icon: Component<{ size?: number }> }[];
		rail?: boolean;
		isActive: (href: string) => boolean;
		label: string;
		onnavigate?: () => void;
	} = $props();
</script>

<nav aria-label={label} class="nomi-nav" class:nomi-nav--rail={rail}>
	{#each items as item (item.href)}
		{@const active = isActive(item.href)}
		<a
			href={item.href}
			class="nomi-nav__item"
			class:nomi-nav__item--active={active}
			aria-current={active ? 'page' : undefined}
			onclick={() => onnavigate?.()}
		>
			<span class="nomi-nav__indicator"><item.icon size={22} /></span>
			<span class="nomi-nav__label">{item.label}</span>
		</a>
	{/each}
</nav>

<style>
	.nomi-nav {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 16px 12px;
	}
	.nomi-nav__item {
		display: flex;
		align-items: center;
		gap: 12px;
		height: 52px;
		padding: 0 16px 0 4px;
		border-radius: var(--md-sys-shape-corner-full);
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: none;
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: 0.9375rem;
		font-weight: 600;
		transition: background-color var(--nomi-motion-effects-fast);
	}
	.nomi-nav__item:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 6%, transparent);
	}
	.nomi-nav__indicator {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 56px;
		height: 32px;
		border-radius: var(--md-sys-shape-corner-full);
		transition:
			background-color var(--nomi-motion-effects-fast),
			width var(--nomi-motion-spatial-fast);
	}
	.nomi-nav__item--active {
		color: var(--md-sys-color-on-surface);
		font-weight: 700;
	}
	.nomi-nav__item--active .nomi-nav__indicator {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}

	/* Collapsed: an M3 navigation rail — icon in its pill, label underneath. */
	.nomi-nav--rail {
		align-items: center;
		gap: 12px;
		padding: 20px 0;
	}
	.nomi-nav--rail .nomi-nav__item {
		flex-direction: column;
		gap: 4px;
		height: auto;
		padding: 0;
		font-size: 0.75rem;
	}
	.nomi-nav--rail .nomi-nav__item:hover {
		background: transparent;
	}
	.nomi-nav--rail .nomi-nav__item:hover .nomi-nav__indicator {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.nomi-nav--rail .nomi-nav__item--active:hover .nomi-nav__indicator {
		background: var(--md-sys-color-primary-container);
	}

</style>
