<!-- What a workspace shares, and what it never does. -->
<script lang="ts">
	import WorkspaceBoundary from '$lib/ds/settings/WorkspaceBoundary.svelte';
</script>

<div class="w-58">
	<WorkspaceBoundary
		shared={{ title: 'Shared here', items: ['Projects and their chats', 'Koda’s builds and deploys'] }}
		personal={{ title: 'Always personal', items: ['Your chats outside projects', 'Memory and profile', 'Inbox'] }}
		tip="Switch where projects come from in the sidebar’s Build group."
	/>
</div>
