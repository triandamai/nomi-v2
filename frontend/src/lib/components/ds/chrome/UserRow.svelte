<script lang="ts">
	import { Settings } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import IconButton from '../actions/IconButton.svelte';
	import Avatar from '../identity/Avatar.svelte';

	/** The person, at the foot of the drawer, with the way into settings. */
	let { name, avatar, settingsHref }: { name: string; avatar?: string; settingsHref: string } = $props();
</script>

<div class="user">
	<Avatar {name} src={avatar} />
	<span class="name">{name}</span>
	<a href={settingsHref} aria-label={m.nav_settings()} class="settings"><Settings size={20} strokeWidth={1.5} aria-hidden="true" /></a>
</div>

<style>
	.user {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 4px 4px 4px 12px;
	}
	.name {
		min-width: 0;
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: var(--md-sys-color-on-surface);
		font: var(--md-sys-typescale-label-large-weight) var(--md-sys-typescale-label-large-size) / var(--md-sys-typescale-label-large-line-height) var(--md-sys-typescale-label-large-font);
	}
	.settings {
		display: inline-flex;
		width: 48px;
		height: 48px;
		align-items: center;
		justify-content: center;
		border-radius: var(--md-sys-shape-corner-full);
		color: var(--md-sys-color-on-surface-variant);
	}
	.settings:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.settings:focus-visible {
		outline: 3px solid var(--md-sys-color-primary);
	}
</style>
