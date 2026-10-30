<script lang="ts">
	import { ArrowUpRight, Bell, GitBranch, RotateCcw } from '@lucide/svelte';
	import MessageUser from '$lib/ds/conversation/MessageUser.svelte';
	import MessageAgent from '$lib/ds/conversation/MessageAgent.svelte';
	import ErrorCallout from '$lib/ds/conversation/ErrorCallout.svelte';
	import ActivityBlock from '$lib/ds/conversation/ActivityBlock.svelte';
	import LiveStep from '$lib/ds/conversation/LiveStep.svelte';
	import EventRow from '$lib/ds/conversation/EventRow.svelte';
	import FileRow from '$lib/ds/conversation/FileRow.svelte';
	import KVRow from '$lib/ds/conversation/KVRow.svelte';
	import ProjectRow from '$lib/ds/lists/ProjectRow.svelte';
	import SuggestionRow from '$lib/ds/lists/SuggestionRow.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import SkeletonRow from '$lib/ds/lists/SkeletonRow.svelte';
	import GallerySection from './GallerySection.svelte';
</script>

{#snippet board(title: string, content: import('svelte').Snippet)}
	<div class="flex flex-col gap-4 rounded-md border border-border p-5">
		<p class="text-ui font-medium text-text">{title}</p>
		{@render content()}
	</div>
{/snippet}

{#snippet messages()}
	<div class="flex max-w-180 flex-col gap-5">
		<MessageUser text="hey" />
		<MessageAgent
			agent="nomi"
			name="Nomi"
			meta="14:02 · deepseek-flash"
			text="I couldn't reply because your DeepSeek account is out of credits for this request. Add credits or raise the API key's limit with DeepSeek, then send your message again."
		>
			<ErrorCallout
				title="Provider error"
				code="402 insufficient_balance"
				requestId="req_9d41e0b2"
				details={[
					{ key: 'reason', value: 'Insufficient Balance', strong: true },
					{ key: 'provider', value: 'api.deepseek.com' },
				]}
				actions={[
					{ label: 'Retry', icon: RotateCcw, onclick: () => {} },
					{ label: 'Add credits on DeepSeek', icon: ArrowUpRight, href: 'https://platform.deepseek.com', external: true },
				]}
			/>
		</MessageAgent>
	</div>
{/snippet}

{#snippet activity()}
	<!-- The pen's Agent activity board, one column: the block, a File Row on its own (400 wide), the live step, an event, a KV row. -->
	<div class="flex max-w-170 flex-col gap-4">
		<ActivityBlock
			title="Wrote 3 files"
			total="+424"
			files={[
				{ path: 'src/routes/+page.svelte', change: '+214' },
				{ path: 'src/lib/prayer-times.ts', change: '+162' },
				{ path: 'src/lib/storage.ts', change: '+48' },
			]}
		/>
		<div data-file-row class="w-100 max-w-full"><FileRow path="src/routes/+page.svelte" change="+214" /></div>
		<LiveStep step="Writing src/routes/menu/+page.svelte" elapsed="01:12" stopLabel="Stop" onstop={() => {}} />
		<EventRow kind="deploy" text="deploy #14 · main@a3f9c21 · built in 41s" time="15:38" />
		<KVRow icon={GitBranch} label="Code" value="23 files · main@a3f9c21" />
	</div>
{/snippet}

{#snippet lists()}
	<div class="flex max-w-240 flex-col gap-4">
		<ProjectRow
			href="#salah"
			name="salah-daily-tracker"
			description="Daily salah tracker with countdown and streaks"
			tone="nomi"
			status={{ kind: 'live', label: 'Live' }}
			url="salah-daily-tracker-0ab8f8.trian.space"
			updated="2d"
		/>
		<SuggestionRow agent="dana" label="Summarize this week's spending" route="→ dana" />
		<div class="flex flex-wrap gap-4">
			<div class="w-80 max-w-full"><EmptyState icon={Bell} title="Reminders" count={0} body="Nothing scheduled today. Ask Kala, or add one yourself." action={{ label: 'Add reminder', href: '#reminders' }} /></div>
			<div class="w-80 max-w-full"><SkeletonRow label="loading 4 projects…" /></div>
		</div>
	</div>
{/snippet}

<GallerySection title="Conversation & build" description="The product-specific parts. Agent activity is legible, never theatrical: real tool, real target, real time.">
	{@render board('Messages', messages)}
	{@render board('Agent activity', activity)}
	{@render board('Lists & empty states', lists)}
</GallerySection>
