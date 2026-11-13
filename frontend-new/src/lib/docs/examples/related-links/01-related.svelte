<!-- Related places. -->
<script lang="ts">
	import { Inbox } from '@lucide/svelte';
	import RelatedLinks from '$lib/ds/settings/RelatedLinks.svelte';
</script>

<div class="w-58">
	<RelatedLinks label="Related" links={[{ href: '/inbox', label: 'Inbox', icon: Inbox }]} />
</div>
