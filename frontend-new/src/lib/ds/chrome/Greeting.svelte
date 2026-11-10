<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';

	// App / Home → Greeting (Z7EVR): Nomi's 32px mascot beside the page's heading in the display
	// size ("Morning, Mario.") and a line on what to do, 16 apart; heading and line 4 apart.
	type Props = { title: string; intro: string };
	let { title, intro }: Props = $props();
</script>

<div class="flex items-center gap-4">
	<Mascot agent="nomi" size={32} />
	<div class="flex min-w-0 flex-col gap-1">
		<h1 class="text-display font-medium text-text">{title}</h1>
		<p class="text-body text-text-2">{intro}</p>
	</div>
</div>
