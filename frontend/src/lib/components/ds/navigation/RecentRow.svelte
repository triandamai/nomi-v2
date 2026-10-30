<script lang="ts">
	import { Folder, MessageSquare } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import { rowStyle } from './rowStyle';

	// A recent chat or project in the drawer, with how long ago in mono.
	type Props = { href: string; kind: 'chat' | 'project'; title: string; when: string; active?: boolean };
	let { href, kind, title, when, active = false }: Props = $props();

	const style = rowStyle();
</script>

<a {href} aria-current={active ? 'page' : undefined} class="recent" class:recent--dense={style.dense}>
	<span class="icon"><Icon icon={kind === 'chat' ? MessageSquare : Folder} size={18} /></span>
	<span class="title">{title}</span>
	<span class="when">{when}</span>
</a>

<style>
	.recent {
		display: flex;
		width: 100%;
		height: 44px;
		align-items: center;
		gap: 12px;
		padding: 0 16px;
		border-radius: var(--md-sys-shape-corner-full);
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: none;
		font: var(--md-sys-typescale-body-medium-weight) var(--md-sys-typescale-body-medium-size) / var(--md-sys-typescale-body-medium-line-height) var(--md-sys-typescale-body-medium-font);
	}
	.recent--dense {
		padding: 0 12px;
	}
	.recent:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
		color: var(--md-sys-color-on-surface);
	}
	.recent:focus-visible {
		outline: 3px solid var(--md-sys-color-primary);
		outline-offset: -3px;
	}
	.recent[aria-current='page'] {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.icon {
		display: inline-flex;
		color: var(--md-sys-color-outline);
	}
	.title {
		min-width: 0;
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.when {
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
	}
</style>
