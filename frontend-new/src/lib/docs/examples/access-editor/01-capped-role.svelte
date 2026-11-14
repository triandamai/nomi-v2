<!-- Capped at the editor's own: levels above what the editor holds can't be chosen, and an area held above it can't be changed. -->
<script lang="ts">
	import AccessEditor from '$lib/ds/admin/AccessEditor.svelte';

	let role = $state<Record<string, 'view' | 'write' | 'manage' | null>>({ plans: 'write', support: 'manage' });
</script>

<div class="w-full max-w-160">
	<AccessEditor
		label="Admin access for ana@example.com"
		bind:value={role}
		max={{ plans: 'manage', notifications: 'write', support: 'write' }}
		areas={[
			{ key: 'plans', label: 'Plans', description: 'The plans people can be on.' },
			{ key: 'notifications', label: 'Notifications', description: 'Broadcasts to everyone’s inbox.' },
			{ key: 'support', label: 'Support', description: 'Articles, drafts, keys and questions.' },
		]}
		levelLabels={{ none: 'None', view: 'View', write: 'Write', manage: 'Manage' }}
		lockedNote="They have more here than you do, so you can’t change it."
	/>
</div>
