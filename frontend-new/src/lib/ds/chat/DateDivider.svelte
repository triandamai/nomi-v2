<script lang="ts">
	// Chat responses → Date Divider: a quiet break between days ("Yesterday", "Mon 27 Oct").
	let { label }: { label: string } = $props();
</script>

<div role="separator" aria-label={label} class="flex w-full items-center gap-2.5">
	<span data-line class="h-px flex-1 bg-border"></span>
	<span aria-hidden="true" class="font-mono text-label tracking-label text-text-3 uppercase">{label}</span>
	<span class="h-px flex-1 bg-border"></span>
</div>
