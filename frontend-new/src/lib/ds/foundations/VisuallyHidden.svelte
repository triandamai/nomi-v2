<script lang="ts">
	/** Text only screen readers announce: a count's meaning, an icon-only control's state. */
	let { text }: { text: string } = $props();
</script>

<span class="sr-only">{text}</span>
