<!-- frontend/src/lib/components/blocks/ContentBlockView.svelte -->
<script lang="ts">
	import type { Component } from 'svelte';
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
	import RunFailedCard from './RunFailedCard.svelte';
	import SuggestionsBlock from './SuggestionsBlock.svelte';
	import QuestionsCard from './QuestionsCard.svelte';
	import VoiceBlock from './VoiceBlock.svelte';
	import ViewBlock from '../views/ViewBlock.svelte';
	import type { ContentBlock } from '$lib/types';

	let {
		block,
		messageId,
		agent = null,
		fresh = false,
		answered = false,
		text = '',
		onremove
	}: {
		block: ContentBlock;
		messageId: string;
		agent?: string | null;
		/** The message arrived just now rather than loading from history. */
		fresh?: boolean;
		/** The person has replied since this message. */
		answered?: boolean;
		/** The message's own text, for blocks that show it (an error bubble). */
		text?: string;
		/** The block took its message away (a failed run, tried again). */
		onremove?: () => void;
	} = $props();

	// Which component draws each kind of block. Each gets the same props (the block and where it
	// sits) and takes the ones it needs. A new kind is a component and a line here; a kind this
	// build doesn't know shows nothing (the message's text stands in).
	const BLOCKS: Partial<Record<ContentBlock['kind'], Component<any>>> = {
		todo_list: TodoListBlock,
		table: TableBlock,
		approval_request: ApprovalCard,
		plan: PlanBlock,
		reminder: ReminderBlock,
		workspace_connect: WorkspaceConnectCard,
		connector_connect: ConnectorConnectCard,
		project_link: ProjectLinkCard,
		questions: QuestionsCard,
		quota_notice: QuotaNoticeCard,
		run_failed: RunFailedCard,
		suggestions: SuggestionsBlock,
		voice: VoiceBlock,
		view: ViewBlock
	};
	const Block = $derived(BLOCKS[block.kind]);

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
{:else if block.kind === 'reasoning'}
	<ReasoningDisclosure steps={[block.text]} />
{:else if Block}
	<Block {block} {messageId} {agent} {fresh} {answered} {text} {onremove} />
{/if}

<style>
	.file-batch {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
</style>
