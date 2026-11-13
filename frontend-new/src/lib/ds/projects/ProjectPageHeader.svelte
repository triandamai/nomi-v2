<script lang="ts">
	import { m } from '$lib/paraglide/messages';

	// Design System / Projects → Project / Page Header (lAF6T): the page title (display) over what
	// projects are (15 at 1.5, text-2), 6 apart, always two grid columns (784) wide.
	type Props = { title?: string; lede?: string };
	let { title = m.projects_title(), lede = m.projects_lede() }: Props = $props();
</script>

<header class="flex w-196 max-w-full flex-col gap-1.5">
	<h1 class="text-display font-medium tracking-display text-text">{title}</h1>
	<p class="text-body leading-normal text-text-2">{lede}</p>
</header>
