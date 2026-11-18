<script lang="ts">
	import { Unplug } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import PhoneButton from '../mobile/PhoneButton.svelte';

	// Flow / Mobile Chat — Errors & recovery, scenario 5 step 3 (UTrnr): a reply the connection cut
	// off part-way. What was written stays above it; this says when it stopped, on warning-bg with
	// 14 corners, [14], and offers Start over and Continue (which finishes it in place).
	type Props = { seconds: number; onrestart?: () => void; oncontinue: () => void };
	let { seconds, onrestart, oncontinue }: Props = $props();
</script>

<div role="status" class="flex w-full flex-col gap-2.5 rounded-xl bg-warning-bg p-3.5">
	<p class="flex items-center gap-2 text-body font-medium text-text">
		<span class="inline-flex shrink-0 text-warning"><Icon icon={Unplug} /></span>
		{m.phone_net_cut({ seconds: String(seconds) })}
	</p>
	<div class="flex gap-2">
		{#if onrestart}<PhoneButton variant="secondary" label={m.reply_start_over()} onclick={onrestart} />{/if}
		<PhoneButton label={m.reply_continue()} onclick={oncontinue} />
	</div>
</div>
