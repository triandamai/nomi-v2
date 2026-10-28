<script lang="ts">
	/** What the person said: on the right, in a surface-2 bubble. */
	let { text }: { text: string } = $props();
</script>

<div class="flex w-full justify-end">
	<p class="max-w-xl rounded-md bg-surface-2 px-3.5 py-2 text-body whitespace-pre-wrap text-text">{text}</p>
</div>
