<!-- Today: a finished reminder, an update, the now line, then what’s still ahead with its tools. -->
<script lang="ts">
	import { Check, Ellipsis, ExternalLink } from '@lucide/svelte';
	import IconButton from '$lib/ds/actions/IconButton.svelte';
	import InboxRow from '$lib/ds/inbox/InboxRow.svelte';
	import NowLine from '$lib/ds/inbox/NowLine.svelte';
</script>

<ul class="flex w-full max-w-215 flex-col border-t border-border">
	<InboxRow time="06:30" agent="kala" title="Gym" tag="reminder · every Mon" caption="Done" finished />
	<InboxRow time="07:02" agent="koda" title="Deployed salah-daily-tracker" tag="deploy" caption="deploy #15 · built in 38s">
		{#snippet tools()}<IconButton icon={ExternalLink} label="Open" variant="ghost" iconSize={14} />{/snippet}
	</InboxRow>
	<li><NowLine time="08:14" label="now" /></li>
	<InboxRow time="16:00" agent="kala" title="Stretch" tag="reminder · once" caption="in 7h 46m" upcoming>
		{#snippet tools()}
			<IconButton icon={Check} label="Mark done" variant="ghost" iconSize={14} />
			<IconButton icon={Ellipsis} label="More for Stretch" variant="ghost" iconSize={14} />
		{/snippet}
	</InboxRow>
</ul>
