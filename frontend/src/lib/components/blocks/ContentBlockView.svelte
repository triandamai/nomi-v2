<!-- frontend/src/lib/components/blocks/ContentBlockView.svelte -->
<script lang="ts">
	import TodoListBlock from './TodoListBlock.svelte';
	import TableBlock from './TableBlock.svelte';
	import ApprovalCard from './ApprovalCard.svelte';
	import PlanBlock from './PlanBlock.svelte';
	import ReasoningDisclosure from './ReasoningDisclosure.svelte';
	import ReminderBlock from './ReminderBlock.svelte';
	import WorkspaceConnectCard from './WorkspaceConnectCard.svelte';
	import ConnectorConnectCard from './ConnectorConnectCard.svelte';
	import ProjectLinkCard from './ProjectLinkCard.svelte';
	import QuotaNoticeCard from './QuotaNoticeCard.svelte';
	import QuestionsCard from './QuestionsCard.svelte';
	import VoiceBlock from './VoiceBlock.svelte';
	import type { ContentBlock } from '$lib/types';

	let {
		block,
		messageId,
		agent = null,
		fresh = false,
		answered = false
	}: {
		block: ContentBlock;
		messageId: string;
		agent?: string | null;
		/** The message arrived just now rather than loading from history. */
		fresh?: boolean;
		/** The person has replied since this message. */
		answered?: boolean;
	} = $props();

	// File changes bring the code editor (CodeMirror): it loads the first time one is shown, not
	// with every chat.
	const fileChangeBlock = () => import('./FileChangeBlock.svelte');
</script>

{#if block.kind === 'file_write' || block.kind === 'file_delete'}
	{#await fileChangeBlock() then { default: FileChangeBlock }}
		<FileChangeBlock {block} {fresh} />
	{/await}
{:else if block.kind === 'file_writes'}
	<!-- Every file of a batch, each its own tab; only a short batch opens by itself. -->
	<div class="file-batch">
		{#await fileChangeBlock() then { default: FileChangeBlock }}
			{#each block.files as file (file.path)}
				<FileChangeBlock block={{ kind: 'file_write', project_id: block.project_id, ...file }} fresh={fresh && block.files.length <= 2} />
			{/each}
		{/await}
	</div>
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
{:else if block.kind === 'connector_connect'}
	<ConnectorConnectCard {block} />
{:else if block.kind === 'project_link'}
	<ProjectLinkCard {block} />
{:else if block.kind === 'questions'}
	<QuestionsCard {block} {agent} {answered} />
{:else if block.kind === 'quota_notice'}
	<QuotaNoticeCard />
{:else if block.kind === 'voice'}
	<VoiceBlock {block} />
{:else if block.kind === 'reasoning'}
	<ReasoningDisclosure steps={[block.text]} />
{/if}

<style>
	.file-batch {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
</style>
