<!-- frontend/src/lib/components/blocks/ContentBlockView.svelte -->
<script lang="ts">
	import FileWriteBlock from './FileWriteBlock.svelte';
	import FileDeleteBlock from './FileDeleteBlock.svelte';
	import TodoListBlock from './TodoListBlock.svelte';
	import TableBlock from './TableBlock.svelte';
	import ApprovalCard from './ApprovalCard.svelte';
	import PlanBlock from './PlanBlock.svelte';
	import ReasoningDisclosure from './ReasoningDisclosure.svelte';
	import ReminderBlock from './ReminderBlock.svelte';
	import WorkspaceConnectCard from './WorkspaceConnectCard.svelte';
	import ProjectLinkCard from './ProjectLinkCard.svelte';
	import type { ContentBlock } from '$lib/types';

	let { block, messageId, agent = null }: { block: ContentBlock; messageId: string; agent?: string | null } = $props();
</script>

{#if block.kind === 'file_write'}
	<FileWriteBlock {block} />
{:else if block.kind === 'file_delete'}
	<FileDeleteBlock {block} />
{:else if block.kind === 'todo_list'}
	<TodoListBlock {block} {agent} />
{:else if block.kind === 'table'}
	<TableBlock {block} />
{:else if block.kind === 'approval_request'}
	<ApprovalCard {block} {messageId} />
{:else if block.kind === 'plan'}
	<PlanBlock {block} {agent} />
{:else if block.kind === 'reminder'}
	<ReminderBlock {block} />
{:else if block.kind === 'workspace_connect'}
	<WorkspaceConnectCard {block} />
{:else if block.kind === 'project_link'}
	<ProjectLinkCard {block} />
{:else if block.kind === 'reasoning'}
	<ReasoningDisclosure steps={[block.text]} />
{/if}
