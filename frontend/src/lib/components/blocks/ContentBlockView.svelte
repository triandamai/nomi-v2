<!-- frontend/src/lib/components/blocks/ContentBlockView.svelte -->
<script lang="ts">
	import type { Component } from 'svelte';
	import TodoListBlock from './TodoListBlock.svelte';
	import TableTwin from './twins/TableTwin.svelte';
	import ApprovalTwin from './twins/ApprovalTwin.svelte';
	import PlanBlock from './PlanBlock.svelte';
	import ReminderTwin from './twins/ReminderTwin.svelte';
	import Reasoning from '$lib/components/ds/chat/Reasoning.svelte';
	import ConnectTwin from './twins/ConnectTwin.svelte';
	import ProjectLinkCard from './ProjectLinkCard.svelte';
	import QuotaNoticeCard from './QuotaNoticeCard.svelte';
	import RunFailedTwin from './twins/RunFailedTwin.svelte';
	import SuggestionsTwin from './twins/SuggestionsTwin.svelte';
	import { m } from '$lib/paraglide/messages';
	import QuestionsTwin from './twins/QuestionsTwin.svelte';
	import VoiceBlock from './VoiceBlock.svelte';
	import StoppedBlock from './StoppedBlock.svelte';
	import ProjectEventBlock from './ProjectEventBlock.svelte';
	import JoinedBlock from './JoinedBlock.svelte';
	import ViewBlock from '../views/ViewBlock.svelte';
	import type { ContentBlock } from '$lib/types';
	import ActivityBlock from '$lib/components/ds/conversation/ActivityBlock.svelte';
	import FileRow from '$lib/components/ds/conversation/FileRow.svelte';
	import { page } from '$app/state';
	import { changeLabel, lineChange } from '$lib/fileChanges';

	let {
		block,
		messageId,
		agent = null,
		fresh = false,
		answered = false,
		text = '',
		time = '',
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
		/** When the message came, "15:38", for blocks that log it (an event row). */
		time?: string;
		/** The block took its message away (a failed run, tried again). */
		onremove?: () => void;
	} = $props();

	// Which component draws each kind of block. Each gets the same props (the block and where it
	// sits) and takes the ones it needs. A new kind is a component and a line here; a kind this
	// build doesn't know shows nothing (the message's text stands in).
	const BLOCKS: Partial<Record<ContentBlock['kind'], Component<any>>> = {
		todo_list: TodoListBlock,
		table: TableTwin,
		approval_request: ApprovalTwin,
		plan: PlanBlock,
		reminder: ReminderTwin,
		workspace_connect: ConnectTwin,
		connector_connect: ConnectTwin,
		project_link: ProjectLinkCard,
		questions: QuestionsTwin,
		quota_notice: QuotaNoticeCard,
		run_failed: RunFailedTwin,
		suggestions: SuggestionsTwin,
		voice: VoiceBlock,
		stopped: StoppedBlock,
		project_event: ProjectEventBlock,
		member_joined: JoinedBlock,
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
	<!-- What was written. In a project: how many files, then each file's line (its check and its
	     change) with its code right under it, opening on a tap. In a chat: the batch summed up
	     (ActivityBlock), then every file's tab. Only a short batch opens by itself. -->
	{@const changes = block.files.map((file) => lineChange(file.content, file.previous_content))}
	{@const inProject = page.url.pathname.startsWith('/projects/')}
	<div class="file-batch">
		{#if inProject}
			<header class="file-batch__head">
				<span class="file-batch__label">{m.files_title()}</span>
				<span>{m.files_written({ done: String(block.files.length), total: String(block.files.length) })}</span>
			</header>
		{:else}
			<ActivityBlock
				title={m.activity_wrote_files({ count: block.files.length })}
				total={changeLabel({ added: changes.reduce((sum, c) => sum + c.added, 0), removed: changes.reduce((sum, c) => sum + c.removed, 0) })}
				files={block.files.map((file, index) => ({ path: file.path, change: changeLabel(changes[index]) }))}
			/>
		{/if}
		{#await fileChangeBlock() then { default: FileChangeBlock }}
			{#each block.files as file, index (file.path)}
				{#if inProject}
					<div class="file-batch__file">
						<FileRow path={file.path} change={changeLabel(changes[index])} state="done" />
						<FileChangeBlock block={{ kind: 'file_write', project_id: block.project_id, ...file }} fresh={fresh && block.files.length <= 2} />
					</div>
				{:else}
					<FileChangeBlock block={{ kind: 'file_write', project_id: block.project_id, ...file }} fresh={fresh && block.files.length <= 2} />
				{/if}
			{/each}
		{/await}
	</div>
{:else if block.kind === 'reasoning'}
	<Reasoning label={m.reasoning_process()} text={block.text} />
{:else if Block}
	<Block {block} {messageId} {agent} {fresh} {answered} {text} {time} {onremove} />
{/if}

<style>
	.file-batch {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.file-batch__head {
		display: flex;
		justify-content: space-between;
		padding: 0 12px;
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
	}
	.file-batch__label {
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	.file-batch__file {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
</style>
