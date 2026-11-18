<script lang="ts">
	// Flow / Mobile Chat — Approvals & background, step 3 (upMaX): what the person might say next,
	// on a phone: each a full-width pill 44 tall, [0,16], at 15, 8 apart. Picking one sends it.
	type Props = { replies: string[]; label?: string; onpick: (reply: string) => void };
	let { replies, label, onpick }: Props = $props();
</script>

<div role="group" aria-label={label} class="flex w-full flex-col gap-2">
	{#each replies as reply (reply)}
		<button type="button" class="flex h-11 w-full cursor-pointer items-center rounded-sheet border border-border-strong px-3.75 text-left text-body text-text outline-offset-2" onclick={() => onpick(reply)}>{reply}</button>
	{/each}
</div>
