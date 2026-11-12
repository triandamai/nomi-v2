<script lang="ts">
	// App / Settings / General (m7lvY1) → On This Page: the page's groups as links down a rule, the
	// one in view marked with the text colour and a 2px stroke.
	type Props = { label: string; links: { href: string; label: string }[]; active?: string };
	let { label, links, active }: Props = $props();
</script>

<nav aria-label={label} class="flex flex-col gap-0.5">
	<p class="pb-1.5 font-mono text-label tracking-label text-text-3 uppercase">{label}</p>
	{#each links as link (link.href)}
		{@const current = link.href === active}
		<a
			href={link.href}
			aria-current={current ? 'location' : undefined}
			class={[
				'py-1.25 text-ui outline-offset-1 hover:text-text',
				current ? 'border-l-2 border-text pl-2 font-medium text-text' : 'border-l border-border pl-2.25 text-text-3',
			]}
		>
			{link.label}
		</a>
	{/each}
</nav>
