<!-- Featured: a live project with its tools. -->
<script lang="ts">
	import { GripVertical } from '@lucide/svelte';
	import IconButton from '$lib/ds/actions/IconButton.svelte';
	import ShowcaseCard from '$lib/ds/profile/ShowcaseCard.svelte';
</script>

<div class="w-full max-w-80">
	<ShowcaseCard name="salah-daily-tracker" status="Live" meta="built with Koda · 4m 12s">
		{#snippet tools()}<IconButton icon={GripVertical} label="Move or remove salah-daily-tracker" variant="ghost" />{/snippet}
	</ShowcaseCard>
</div>
