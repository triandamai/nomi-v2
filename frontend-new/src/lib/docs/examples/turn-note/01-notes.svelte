<!-- Notes: a recovered failure folded into a row, and quiet lines. -->
<script lang="ts">
	import { CircleCheck, LogIn, Unplug } from '@lucide/svelte';
	import TurnNote from '$lib/ds/thread/TurnNote.svelte';
</script>

<div class="flex w-full max-w-180 flex-col gap-4">
	<TurnNote kind="box" icon={CircleCheck} text="First try failed: DeepSeek is having trouble right now. Retried." action={{ label: 'Details', onclick: () => {} }} />
	<TurnNote icon={Unplug} text="continued after the connection dropped at 18s" />
	<TurnNote icon={LogIn} align="end" text="sent after you signed in · 10:31" />
</div>
