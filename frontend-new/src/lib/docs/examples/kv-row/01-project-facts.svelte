<!-- Project facts: a short key, and the value as the machine reports it. -->
<script lang="ts">
	import { GitBranch, Globe } from '@lucide/svelte';
	import KVRow from '$lib/ds/conversation/KVRow.svelte';
</script>

<div class="flex w-full max-w-100 flex-col">
	<KVRow icon={GitBranch} label="Code" value="23 files · main@a3f9c21" />
	<KVRow icon={Globe} label="URL" value="salah-daily-tracker-0ab8f8.trian.space" />
</div>
