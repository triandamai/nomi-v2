<!-- Pill states: up to date, changes waiting, publishing, failed, and a member's ask. -->
<script lang="ts">
	import PhoneWorkspaceNav from '$lib/ds/projects/PhoneWorkspaceNav.svelte';
	const nav = { project: 'warung-kopi-menu', status: 'live · deploy #2', live: true, onback: () => {}, onmenu: () => {} };
</script>

<div class="flex w-97.5 max-w-full flex-col gap-2">
	<PhoneWorkspaceNav {...nav} pill={{ kind: 'idle' }} />
	<PhoneWorkspaceNav {...nav} pill={{ kind: 'ready', count: 3, onpublish: () => {} }} />
	<PhoneWorkspaceNav {...nav} pill={{ kind: 'publishing', elapsed: '0:24' }} />
	<PhoneWorkspaceNav {...nav} pill={{ kind: 'failed', onretry: () => {} }} />
	<PhoneWorkspaceNav {...nav} pill={{ kind: 'ask', onask: () => {} }} />
</div>
