<script lang="ts">
	import type { Snippet } from 'svelte';
	import { PanelLeft, Plus } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import IconButton from '../actions/IconButton.svelte';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import { setRowStyle } from '../navigation/rowStyle';

	// Twin of frontend-new's Sidebar as an M3 navigation drawer: brand and collapse, New chat as
	// the drawer's FAB, the sections (children: NavItems and group labels), recents, then usage
	// and the person at the foot.
	type Props = {
		newChatHref: string;
		newChatShortcut?: string;
		oncollapse?: () => void;
		children: Snippet;
		recents?: Snippet;
		footer?: Snippet;
	};
	let { newChatHref, newChatShortcut = '⌘K', oncollapse, children, recents, footer }: Props = $props();

	setRowStyle({ dense: true, icon: 'text-2' });
</script>

<nav aria-label={m.nav_main()} class="drawer">
	<div class="brand">
		<span class="mark">
			<Mascot agent="nomi" size={32} />
			<span class="wordmark">nomi</span>
		</span>
		{#if oncollapse}<IconButton icon={PanelLeft} label={m.nav_collapse()} variant="ghost" onclick={oncollapse} />{/if}
	</div>
	<a href={newChatHref} class="new-chat">
		<Icon icon={Plus} size={20} />
		<span class="new-chat__label">{m.nav_new_chat()}</span>
		<span class="shortcut">{newChatShortcut}</span>
	</a>
	<div class="sections">{@render children()}</div>
	{#if recents}<div class="recents">{@render recents()}</div>{:else}<div class="spacer"></div>{/if}
	{#if footer}<div class="footer">{@render footer()}</div>{/if}
</nav>

<style>
	.drawer {
		display: flex;
		height: 100%;
		width: 300px;
		flex-shrink: 0;
		flex-direction: column;
		gap: 16px;
		padding: 12px;
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface);
	}
	.brand {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 4px 4px 4px 12px;
	}
	.mark {
		display: inline-flex;
		align-items: center;
		gap: 10px;
	}
	.wordmark {
		font: var(--md-sys-typescale-title-large-weight) var(--md-sys-typescale-title-large-size) / var(--md-sys-typescale-title-large-line-height) var(--md-sys-typescale-title-large-font);
	}
	.new-chat {
		display: flex;
		height: 56px;
		align-items: center;
		gap: 12px;
		padding: 0 20px 0 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
		text-decoration: none;
		box-shadow: var(--md-sys-elevation-shadow-level2);
		font: var(--md-sys-typescale-label-large-weight) var(--md-sys-typescale-label-large-size) / var(--md-sys-typescale-label-large-line-height) var(--md-sys-typescale-label-large-font);
	}
	.new-chat:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-primary-container) 8%, var(--md-sys-color-primary-container));
	}
	.new-chat:focus-visible {
		outline: 3px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.new-chat__label {
		flex: 1;
	}
	.shortcut {
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
		opacity: 0.72;
	}
	.sections {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.recents {
		display: flex;
		min-height: 0;
		flex: 1;
		flex-direction: column;
		overflow-y: auto;
	}
	.spacer {
		flex: 1;
	}
	.footer {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
</style>
