<!-- Typing: three digits in, the fourth box waiting. Paste a whole code into any box. -->
<script lang="ts">
	import CodeInput from '$lib/ds/auth/CodeInput.svelte';

	let code = $state('481');
	let sent = $state('');
</script>

<div class="flex w-full max-w-95 flex-col gap-3">
	<CodeInput bind:value={code} oncomplete={(full) => (sent = full)} />
	<p class="font-mono text-label text-text-3">{sent ? `sent ${sent}` : `typed ${code || '—'}`}</p>
</div>
