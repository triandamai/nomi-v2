<script lang="ts">
	// Flow / Mobile — Home, Inbox & You, step 3 (O1SXw) → Plan (a69h1d): the plan and how much of the
	// month's tokens it has used. A card (surface, border, radius 16, inset 14, 10 apart): the plan at
	// 16/500 with the share in mono, a 6-high meter (surface-2, the used part in the accent), and the
	// tokens in mono with Plan & usage.
	type Props = { plan: string; used: number; caption: string; href: string; link: string };
	let { plan, used, caption, href, link }: Props = $props();
	const percent = $derived(Math.round(Math.min(Math.max(used, 0), 1) * 100));
</script>

<section aria-label={plan} class="flex flex-col gap-2.5 rounded-2xl bg-surface p-3.5 outline -outline-offset-1 outline-border">
	<div class="flex items-center gap-2">
		<span class="min-w-0 flex-1 truncate text-phone font-medium text-text">{plan}</span>
		<span class="shrink-0 font-mono text-ui text-text-2">{percent}%</span>
	</div>
	<div role="meter" aria-label={caption} aria-valuemin={0} aria-valuemax={100} aria-valuenow={percent} class="h-1.5 overflow-hidden rounded-full bg-surface-2">
		<div class="h-full rounded-full bg-accent" style:width="{Math.max(percent, used > 0 ? 2 : 0)}%"></div>
	</div>
	<div class="flex items-center gap-2">
		<span class="min-w-0 flex-1 truncate font-mono text-label text-text-3">{caption}</span>
		<a {href} class="shrink-0 rounded-xs text-phone-sm font-medium text-text outline-offset-2 focus-visible:outline-2 focus-visible:outline-focus pointer-coarse:py-3">{link}</a>
	</div>
</section>
