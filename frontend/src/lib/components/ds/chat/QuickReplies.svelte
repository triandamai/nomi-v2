<script lang="ts">
	// Chat responses → Quick Replies: answers to the question just asked, one tap each. Round, so
	// they read as replies rather than actions.
	let { replies, onpick }: { replies: string[]; onpick: (reply: string) => void } = $props();
</script>

<div class="flex flex-wrap items-center gap-2">
	{#each replies as reply (reply)}
		<button
			type="button"
			class="inline-flex h-7 cursor-pointer items-center rounded-full border border-border-strong px-2.75 text-ui text-text outline-offset-1 transition-colors hover:bg-surface-2 pointer-coarse:min-h-11"
			onclick={() => onpick(reply)}>{reply}</button
		>
	{/each}
</div>
