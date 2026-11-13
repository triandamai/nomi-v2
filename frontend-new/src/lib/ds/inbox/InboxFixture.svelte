<script lang="ts">
	import { ArrowUpRight, Check, Ellipsis, SquareTerminal } from '@lucide/svelte';
	import Button from '../actions/Button.svelte';
	import IconButton from '../actions/IconButton.svelte';
	import InboxCard from './InboxCard.svelte';
	import InboxRow from './InboxRow.svelte';

	// For the tests: a card and today's rows, 860 wide as in App / Inbox (BFoiZ).
</script>

<div class="flex w-215 flex-col gap-4">
	<InboxCard time="16 Oct" agent="koda" title="Deploy failed: invoice-sinar-jaya" tag="project" caption="Build error in src/lib/pdf.ts. The last good version is still live." unread>
		{#snippet actions()}
			<Button label="Ask Koda to fix" variant="secondary" iconEnd={ArrowUpRight} />
			<Button label="Open log" variant="ghost" iconStart={SquareTerminal} />
		{/snippet}
	</InboxCard>
	<ul class="flex flex-col">
		<InboxRow data-testid="row-gym" time="06:30" agent="kala" title="Gym" tag="reminder · Mon Wed Fri" caption="Done" finished />
		<InboxRow data-testid="row-stretch" time="16:00" agent="kala" title="Stretch" tag="reminder · weekdays" caption="in 7h 46m" upcoming>
			{#snippet tools()}
				<IconButton icon={Check} label="Mark done" variant="ghost" iconSize={14} />
				<IconButton icon={Ellipsis} label="More" variant="ghost" iconSize={14} />
			{/snippet}
		</InboxRow>
	</ul>
</div>
