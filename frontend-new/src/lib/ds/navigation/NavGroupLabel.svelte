<script lang="ts">
	/** Names a group of sidebar rows (Talk, Build, Life). Written in sentence case; shown in capitals. */
	let { label }: { label: string } = $props();
</script>

<p class="px-2.5 pt-3 pb-1 font-mono text-label tracking-label text-text-3 uppercase">{label}</p>
