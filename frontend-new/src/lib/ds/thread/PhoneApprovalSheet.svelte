<script lang="ts">
	import { Check, Image } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import BottomSheet from '../overlays/BottomSheet.svelte';
	import PhoneButton from '../mobile/PhoneButton.svelte';

	// Flow / Mobile Chat — Plan → approve → act, step 5 (Yi1GU): everything an approval would send,
	// in full, before it goes ("Send this email? · Tara · via your Gmail"). Each field a row 12
	// apart over a hairline (the name at 14 in text-3, 64 wide; the value at 15), the message at 15
	// / 1.55, the attachment in mono on a 12-round outlined row, then the one button. The pen's
	// "Don't ask again" switch and Edit are left out: the backend can't scope the first to a
	// recipient or take edited input yet (UX contract D3, G8).
	type Props = {
		open?: boolean;
		title: string;
		subtitle?: string;
		fields: { key: string; value: string }[];
		body?: string;
		attachment?: string;
		approveLabel: string;
		/** While the decision is on its way. */
		busy?: boolean;
		onapprove: () => void;
		onclose?: () => void;
	};
	let { open = $bindable(false), title, subtitle, fields, body, attachment, approveLabel, busy = false, onapprove, onclose }: Props = $props();
</script>

<BottomSheet bind:open {title} {subtitle} {onclose}>
	<div class="flex flex-col gap-2.5 px-4 pb-2">
		{#each fields as field (field.key)}
			<p class="flex gap-3 border-b border-border pt-2 pb-1.75">
				<span class="w-16 shrink-0 text-phone-sm text-text-3">{field.key}</span>
				<span class="min-w-0 flex-1 text-body break-words text-text">{field.value}</span>
			</p>
		{/each}
		{#if body}<p class="text-body leading-phone whitespace-pre-line text-text">{body}</p>{/if}
		{#if attachment}
			<p class="flex items-center gap-2.5 rounded-lg px-3 py-2.5 outline -outline-offset-1 outline-border">
				<span class="inline-flex shrink-0 text-text-2"><Icon icon={Image} size={18} /></span>
				<span class="min-w-0 flex-1 truncate font-mono text-ui text-text-2">{attachment}</span>
			</p>
		{/if}
		<PhoneButton label={approveLabel} icon={Check} disabled={busy} onclick={onapprove} />
	</div>
</BottomSheet>
