<!-- Wrong code: every box edged in danger, the line under it says how many tries are left. -->
<script lang="ts">
	import CodeInput from '$lib/ds/auth/CodeInput.svelte';
</script>

<div class="flex w-full max-w-95 flex-col gap-3">
	<CodeInput value="123456" invalid describedby="wrong-code-example" />
	<p id="wrong-code-example" class="text-caption text-danger">That code isn’t right. 4 tries left.</p>
</div>
