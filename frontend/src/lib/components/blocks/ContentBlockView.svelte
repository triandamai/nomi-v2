<!-- frontend/src/lib/components/blocks/ContentBlockView.svelte -->
<script lang="ts">
	import FileWriteBlock from './FileWriteBlock.svelte';
	import FileDeleteBlock from './FileDeleteBlock.svelte';
	import TodoListBlock from './TodoListBlock.svelte';
	import TableBlock from './TableBlock.svelte';
	import ApprovalCard from './ApprovalCard.svelte';
	import type { ContentBlock } from '$lib/types';

	let { block, messageId }: { block: ContentBlock; messageId: string } = $props();
</script>

{#if block.kind === 'file_write'}
	<FileWriteBlock {block} />
{:else if block.kind === 'file_delete'}
	<FileDeleteBlock {block} />
{:else if block.kind === 'todo_list'}
	<TodoListBlock {block} />
{:else if block.kind === 'table'}
	<TableBlock {block} />
{:else if block.kind === 'approval_request'}
	<ApprovalCard {block} {messageId} />
{/if}
