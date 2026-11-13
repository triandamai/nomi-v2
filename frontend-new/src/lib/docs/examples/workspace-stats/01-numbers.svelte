<!-- A workspace with four projects, three members and an open invite. -->
<script lang="ts">
	import { Folder, Mail, Users } from '@lucide/svelte';
	import WorkspaceStats from '$lib/ds/settings/WorkspaceStats.svelte';
</script>

<div class="w-full max-w-220">
	<WorkspaceStats label="In this workspace" stats={[{ icon: Folder, value: 4, label: 'Projects' }, { icon: Users, value: 3, label: 'Members' }, { icon: Mail, value: 1, label: 'Pending invites' }]} />
</div>
