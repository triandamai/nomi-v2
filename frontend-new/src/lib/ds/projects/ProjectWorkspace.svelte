<script lang="ts">
	import type { Snippet } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import Segmented from '../inputs/Segmented.svelte';
	import ResizeHandle from '../build/ResizeHandle.svelte';

	// Flow / Build → Screen / Building (H4DIH) → a project's workspace: the project bar (the caller's
	// own, 52), the agent chat (520 as the flow draws it, dragged between 360 and 720 by the resize
	// handle, the preview never under 480), then the preview pane. On a phone, Chat and Preview
	// switch at the top. The shells' WorkspaceShell (a twin of the old app's, 440 and fixed) stays as
	// it is; this is the workspace as Flow / Build has it.
	type Props = { projectbar: Snippet; chat: Snippet; children: Snippet; view?: 'chat' | 'pane'; chatWidth?: number };
	let { projectbar, chat, children, view = $bindable('chat'), chatWidth = $bindable(520) }: Props = $props();

	let room = $state(0);
</script>

<div class="flex h-dvh w-full flex-col overflow-hidden bg-bg text-text">
	{@render projectbar()}
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
	<div bind:clientWidth={room} class="flex min-h-0 flex-1" style="--chat-width: {chatWidth}px">
		<section aria-label={m.workspace_chat()} class={['min-h-0 flex-col lg:flex lg:w-(--chat-width) lg:shrink-0', view === 'chat' ? 'flex w-full' : 'hidden']}>
			{@render chat()}
		</section>
		<div class="hidden lg:flex"><ResizeHandle width={chatWidth} available={room} onresize={(next) => (chatWidth = next)} /></div>
		<div class={['min-h-0 min-w-0 flex-1 flex-col lg:flex', view === 'pane' ? 'flex' : 'hidden']}>{@render children()}</div>
	</div>
</div>
