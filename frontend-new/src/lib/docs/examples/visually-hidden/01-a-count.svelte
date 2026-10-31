<!-- A count's meaning: people see "3"; screen readers hear "3 new". -->
<script lang="ts">
	import VisuallyHidden from '$lib/ds/foundations/VisuallyHidden.svelte';
</script>

<p class="flex items-center gap-3 text-ui text-text">
	Inbox
	<span class="font-mono text-label text-text-3">3<VisuallyHidden text=" new" /></span>
</p>
