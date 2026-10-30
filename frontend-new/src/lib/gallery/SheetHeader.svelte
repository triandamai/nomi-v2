<script lang="ts">
	/**
	 * A design sheet's DS Header, as the pen draws it on Build and Chat responses: the mono eyebrow
	 * (written as the pen has it), the title (40px on the Chat sheet: `hero`) and an 880-wide lede,
	 * 8px apart.
	 */
	type Props = { eyebrow: string; title: string; lede: string; hero?: boolean };
	let { eyebrow, title, lede, hero = false }: Props = $props();
</script>

<header class="flex flex-col gap-2">
	<p class="font-mono text-label tracking-label text-text-3">{eyebrow}</p>
	<h1 class={['font-medium text-text', hero ? 'hero' : 'text-display tracking-display']}>{title}</h1>
	<p class="w-220 max-w-full text-body leading-normal text-text-2">{lede}</p>
</header>

<style>
	/* The Chat sheet's own title size: documentation, not app UI, so not a token. */
	.hero {
		font-size: 40px;
		line-height: 52px;
		letter-spacing: -0.8px;
	}
</style>
