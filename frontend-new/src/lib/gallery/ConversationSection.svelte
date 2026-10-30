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
	import GalleryBoard from './GalleryBoard.svelte';
</script>

{#snippet messages()}
	<MessageUser text="hey" />
	<MessageAgent
		agent="nomi"
		name="Nomi"
		meta="14:02 · deepseek-flash"
		text="I couldn't reply because your DeepSeek account is out of credits for this request. Add credits or raise the API key's limit with DeepSeek, then send your message again."
	/>
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
{/snippet}

{#snippet activity()}
	<!-- The pen's Agent activity board, one column: the block, a File Row on its own (400 wide), the live step, an event, a KV row. -->
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
{/snippet}

{#snippet lists()}
	<div class="flex flex-col gap-6 lg:flex-row lg:items-start">
		<div class="flex min-w-0 flex-1 flex-col gap-4">
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
		</div>
		<div class="w-80 max-w-full shrink-0"><EmptyState icon={Bell} title="Reminders" count={0} body="Nothing scheduled today. Ask Kala, or add one yourself." action={{ label: 'Add reminder', href: '#reminders' }} /></div>
		<div class="w-80 max-w-full shrink-0"><SkeletonRow label="loading 4 projects…" /></div>
	</div>
{/snippet}

<GallerySection title="Conversation & build" description="The product-specific parts. Agent activity is legible, never theatrical: real tool, real target, real time.">
	<div class="flex flex-col gap-6 lg:flex-row lg:items-start">
		<div class="lg:w-180 lg:shrink-0"><GalleryBoard title="Messages" note="" nomi>{@render messages()}</GalleryBoard></div>
		<div class="min-w-0 flex-1"><GalleryBoard title="Agent activity" note="" nomi>{@render activity()}</GalleryBoard></div>
	</div>
	<GalleryBoard title="Lists & empty states" note="" nomi>{@render lists()}</GalleryBoard>
</GallerySection>
