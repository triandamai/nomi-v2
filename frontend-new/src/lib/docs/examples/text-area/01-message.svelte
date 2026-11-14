<!-- A notification's message, with the backend's limit counted. -->
<script lang="ts">
	import TextArea from '$lib/ds/admin/TextArea.svelte';

	let body = $state('Pro is half price until the end of the month.');
</script>

<div class="w-full max-w-120">
	<TextArea label="Message" bind:value={body} maxlength={2000} helper="Shown in everyone’s inbox." />
</div>
