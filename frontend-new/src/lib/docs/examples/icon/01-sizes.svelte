<!-- Sizes: 16 in nav and buttons, 14 inline with text, 12 in chips. -->
<script lang="ts">
	import { Folder, GitBranch, MessageSquare } from '@lucide/svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
</script>

<div class="flex flex-wrap items-center gap-8">
	<span class="inline-flex items-center gap-3 text-ui text-text-2"><Icon icon={MessageSquare} /> Chats</span>
	<span class="inline-flex items-center gap-2 text-ui text-text-2"><Icon icon={Folder} size={14} /> salah-daily-tracker</span>
	<span class="inline-flex items-center gap-1.5 font-mono text-label text-text-2"><Icon icon={GitBranch} size={12} /> git repo</span>
</div>
