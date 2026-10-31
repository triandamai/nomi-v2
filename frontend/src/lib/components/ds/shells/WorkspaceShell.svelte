<script lang="ts">
	import type { Snippet } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import Segmented from '../inputs/Segmented.svelte';

	// The workspace shell (inside one project): a top app bar, the agent chat, then preview / code /
	// terminal. On a phone, Chat and Preview switch at the top instead of sitting side by side.
	type Props = { projectbar: Snippet; chat: Snippet; children: Snippet; view?: 'chat' | 'pane' };
	let { projectbar, chat, children, view = $bindable('chat') }: Props = $props();
</script>

<div class="workspace">
	<header class="bar">{@render projectbar()}</header>
	<div class="switch">
		<Segmented
			label={m.workspace_view()}
			options={[
				{ value: 'chat', label: m.workspace_chat() },
				{ value: 'pane', label: m.workspace_preview() },
			]}
			bind:value={view}
		/>
	</div>
	<div class="panes">
		<section aria-label={m.workspace_chat()} class="chat" class:shown={view === 'chat'}>{@render chat()}</section>
		<div class="pane" class:shown={view === 'pane'}>{@render children()}</div>
	</div>
</div>

<style>
	.workspace {
		display: flex;
		height: 100dvh;
		width: 100%;
		flex-direction: column;
		overflow: hidden;
		background: var(--nomi-color-ground);
		color: var(--md-sys-color-on-surface);
	}
	.bar {
		display: flex;
		height: 64px;
		flex-shrink: 0;
		align-items: center;
		gap: 12px;
		padding: 0 16px;
	}
	.switch {
		display: flex;
		justify-content: center;
		padding: 8px;
	}
	.panes {
		display: flex;
		min-height: 0;
		flex: 1;
		gap: 12px;
		padding: 0 12px 12px;
	}
	.chat,
	.pane {
		display: none;
		min-height: 0;
		min-width: 0;
		flex: 1;
		flex-direction: column;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.shown {
		display: flex;
	}
	@media (min-width: 1024px) {
		.switch {
			display: none;
		}
		.chat,
		.pane {
			display: flex;
		}
		.chat {
			width: 440px;
			flex: none;
		}
	}
</style>
