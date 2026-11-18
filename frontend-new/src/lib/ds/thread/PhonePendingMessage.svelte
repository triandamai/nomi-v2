<script lang="ts">
	import { CircleAlert, Hourglass, LogIn } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile Chat — Errors & recovery, scenarios 5–7 (x8dgV, TJhp2, M75Qz): your message on
	// a phone with what happened to it under the bubble, 4 apart: queued while offline ("queued ·
	// sends when you're back", Cancel), not sent with the reason in danger (the bubble's stroke too,
	// Retry), or sent late ("sent after you signed in · 10:31"). The note is mono at 11, its icon
	// 12, its action at 13 / 500.
	type Props = {
		text: string;
		status: 'queued' | 'failed' | 'late';
		note: string;
		action?: { label: string; onclick: () => void };
	};
	let { text, status, note, action }: Props = $props();
	const ICON = { queued: Hourglass, failed: CircleAlert, late: LogIn } as const;
</script>

<div class="flex w-full flex-col items-end gap-1">
	<p class={['max-w-70 rounded-3xl bg-surface-2 px-3.5 py-2.5 text-phone leading-cozy whitespace-pre-wrap text-text', status === 'failed' && 'outline -outline-offset-1 outline-danger-border']}>{text}</p>
	<p class="flex items-center gap-1.5 px-1">
		<span class={['inline-flex items-center gap-1.5 font-mono text-label', status === 'failed' ? 'text-danger' : 'text-text-3']}><Icon icon={ICON[status]} size={12} />{note}</span>
		{#if action}<button type="button" class="cursor-pointer rounded-xs text-ui font-medium text-text outline-offset-1 pointer-coarse:min-h-11" onclick={action.onclick}>{action.label}</button>{/if}
	</p>
</div>
