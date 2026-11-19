<!-- A file in Code: the path in mono, opening the file. -->
<script lang="ts">
	import { FileCode } from '@lucide/svelte';
	import PhoneList from '$lib/ds/projects/PhoneList.svelte';
	import PhoneListRow from '$lib/ds/projects/PhoneListRow.svelte';
</script>

<div class="w-89.5 max-w-full">
	<PhoneList label="Code"><PhoneListRow icon={FileCode} title="src/routes/menu/+page.svelte" mono onclick={() => {}} /></PhoneList>
</div>
