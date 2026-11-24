<!-- The day: one done, one still to come. -->
<script lang="ts">
	import { Ellipsis } from '@lucide/svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import PhoneTimelineRow from '$lib/ds/inbox/PhoneTimelineRow.svelte';
</script>

<ul class="flex w-97.5 max-w-full flex-col">
	<PhoneTimelineRow time="06:30" title="Gym" meta="reminder · Mon Wed Fri" state="Done 06:42" done />
	<PhoneTimelineRow time="16:00" title="Stretch" meta="reminder · weekdays" state="in 7h 46m">
		{#snippet tools()}
			<button type="button" aria-label="More for Stretch" class="inline-flex size-11 items-center justify-center rounded-full text-text-2"><Icon icon={Ellipsis} size={18} /></button>
		{/snippet}
	</PhoneTimelineRow>
</ul>
