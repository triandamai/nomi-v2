<script lang="ts">
	import type { Snippet } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import Segmented from '../inputs/Segmented.svelte';

	// Layout shells → Workspace shell (inside one project): project bar 52, agent chat 440, then
	// preview / code / terminal. On a phone, Chat and Preview switch at the top instead of sitting
	// side by side.
	type Props = { projectbar: Snippet; chat: Snippet; children: Snippet; view?: 'chat' | 'pane' };
	let { projectbar, chat, children, view = $bindable('chat') }: Props = $props();
</script>

<div class="flex h-dvh w-full flex-col overflow-hidden bg-bg text-text">
	<header class="flex h-13 shrink-0 items-center gap-3 border-b border-border px-4">{@render projectbar()}</header>
	<div class="flex justify-center border-b border-border p-2 lg:hidden">
		<Segmented
			label={m.workspace_view()}
			options={[
				{ value: 'chat', label: m.workspace_chat() },
				{ value: 'pane', label: m.workspace_preview() },
			]}
			bind:value={view}
		/>
	</div>
	<div class="flex min-h-0 flex-1">
		<section aria-label={m.workspace_chat()} class={['min-h-0 flex-col border-border lg:flex lg:w-110 lg:shrink-0 lg:border-r', view === 'chat' ? 'flex w-full' : 'hidden']}>
			{@render chat()}
		</section>
		<div class={['min-h-0 min-w-0 flex-1 flex-col lg:flex', view === 'pane' ? 'flex' : 'hidden']}>{@render children()}</div>
	</div>
</div>
