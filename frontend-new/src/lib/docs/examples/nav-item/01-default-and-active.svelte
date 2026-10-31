<!-- Default and active: the page people are on has the fill and primary text. -->
<script lang="ts">
	import { Folder, House, MessageSquare } from '@lucide/svelte';
	import NavItem from '$lib/ds/navigation/NavItem.svelte';
</script>

<div class="flex w-56 flex-col gap-0.5">
	<NavItem href="#home" icon={House} label="Home" />
	<NavItem href="#chats" icon={MessageSquare} label="Chats" count={12} />
	<NavItem href="#projects" icon={Folder} label="Projects" count={4} active />
</div>
