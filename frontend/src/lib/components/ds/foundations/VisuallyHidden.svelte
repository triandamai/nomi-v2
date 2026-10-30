<script lang="ts">
	/** Text only screen readers announce: a count's meaning, an icon-only control's state. */
	let { text }: { text: string } = $props();
</script>

<span class="visually-hidden">{text}</span>

<style>
	.visually-hidden {
		position: absolute;
		width: 1px;
		height: 1px;
		margin: -1px;
		padding: 0;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border: 0;
	}
</style>
