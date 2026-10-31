<!-- Groups: written in sentence case, shown in capitals. Build carries the workspace chip. -->
<script lang="ts">
	import { Folder, MessageSquare } from '@lucide/svelte';
	import NavGroupLabel from '$lib/ds/navigation/NavGroupLabel.svelte';
	import NavItem from '$lib/ds/navigation/NavItem.svelte';
	import WorkspaceChip from '$lib/ds/chrome/WorkspaceChip.svelte';
</script>

<div class="flex w-56 flex-col gap-0.5">
	<NavGroupLabel label="Talk" />
	<NavItem href="#chats" icon={MessageSquare} label="Chats" count={12} />
	<NavGroupLabel label="Build"><WorkspaceChip name="Trian's space" onclick={() => {}} /></NavGroupLabel>
	<NavItem href="#projects" icon={Folder} label="Projects" count={4} />
</div>
