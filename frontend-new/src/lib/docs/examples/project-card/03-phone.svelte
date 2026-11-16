<!-- Phone: Mobile / Project Card, failed, with its pill and the sheet's ⋯. -->
<script lang="ts">
	import ProjectCard from '$lib/ds/projects/ProjectCard.svelte';
</script>

<div class="w-89.5 max-w-full">
	<ProjectCard
		phone
		name="invoice-sinar-jaya"
		description="Invoices and payment tracking for PT Sinar Jaya"
		tone="kala"
		status="failed"
		meta="deploy #6"
		detail="build error in src/lib/pdf.ts · #5 still live"
		footer="3 chats · 5d ago"
		onaction={() => {}}
		onmore={() => {}}
	/>
</div>
