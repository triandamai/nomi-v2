<!-- States: live, building, failed and waiting on an answer. Each says its one next action. -->
<script lang="ts">
	import ProjectCard from '$lib/ds/projects/ProjectCard.svelte';

	const noop = () => {};
</script>

<div class="flex flex-wrap gap-4">
	<ProjectCard
		name="salah-daily-tracker"
		description="Daily salah tracker with countdown and streaks"
		tone="nomi"
		status="live"
		url="salah-daily-tracker-0ab8f8.trian.space"
		footer="2 chats · deploy #15 · 2d ago"
		href="#open"
		actions={{ onopen: noop, onvisit: noop, onrename: noop, ondelete: noop }}
	/>
	<ProjectCard name="warung-kopi-menu" description="Menu and pre-order page for Warung Kopi Bu Siti" tone="dana" status="building" meta="3 of 6" detail="writing routes/menu/+page.svelte" footer="1 chat · started 08:13" onaction={noop} />
	<ProjectCard name="invoice-sinar-jaya" description="Invoices and payment tracking for PT Sinar Jaya" tone="kala" status="failed" meta="deploy #6" detail="build error in src/lib/pdf.ts · #5 still live" footer="3 chats · 5d ago" onaction={noop} />
	<ProjectCard name="laundry-kiloan" description="Orders and pickup tracker for Laundry Bu Ani" tone="maya" status="attention" questions={3} detail="who uses it · how orders come in · prices" footer="1 chat · started 09:30" onaction={noop} />
</div>
