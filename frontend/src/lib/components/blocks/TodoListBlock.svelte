<!-- frontend/src/lib/components/blocks/TodoListBlock.svelte -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import ChecklistBubble from './ChecklistBubble.svelte';
	import type { ContentBlock } from '$lib/types';

	let { block, agent = null }: { block: Extract<ContentBlock, { kind: 'todo_list' }>; agent?: string | null } = $props();
</script>

<ChecklistBubble {agent} heading={m.todo_heading({ agent: agent ?? 'Nomi' })} items={block.items} />
