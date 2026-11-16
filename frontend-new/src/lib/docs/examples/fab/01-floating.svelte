<!-- FAB: New chat on Chats, New project on Projects. -->
<script lang="ts">
	import { Plus, SquarePen } from '@lucide/svelte';
	import Fab from '$lib/ds/chrome/Fab.svelte';
</script>

<div class="flex gap-6">
	<Fab label="New chat" icon={SquarePen} iconSize={22} href="#new-chat" />
	<Fab label="New project" icon={Plus} onclick={() => {}} />
</div>
