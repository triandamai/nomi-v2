<!-- Google Workspace, connected and used by Tara. -->
<script lang="ts">
	import { Mail } from '@lucide/svelte';
	import ConnectionCard from '$lib/ds/settings/ConnectionCard.svelte';
</script>

<div class="w-96 max-w-full">
	<ConnectionCard
		icon={Mail}
		name="Google Workspace"
		account="ko••@gmail.com"
		status="on"
		caption="Gmail, Docs, Sheets, Drive and Calendar"
		canLabel="Can do"
		can={['Gmail', 'Docs', 'Calendar']}
		usedBy={[{ agent: 'tara', name: 'Tara' }]}
		when="2h ago"
		manage="Manage"
	/>
</div>
