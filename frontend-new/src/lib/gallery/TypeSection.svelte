<script lang="ts">
	import GallerySection from './GallerySection.svelte';

	const styles = [
		{ token: 'nm-size-display', spec: '28px · Geist 500', sample: 'Morning, Mario.', use: 'Page titles, greeting', class: 'text-display font-medium tracking-display' },
		{ token: 'nm-size-body', spec: '15px · Geist 400', sample: "I couldn't reply because your DeepSeek account is out of credits.", use: 'Message body, page intro', class: 'text-body' },
		{ token: 'nm-size-ui', spec: '13px · Geist 500', sample: 'Open workspace', use: 'Buttons, nav, rows, labels', class: 'text-ui font-medium' },
		{ token: 'nm-size-caption', spec: '12px · Geist 400', sample: 'Transactions, budgets, subscriptions', use: 'Descriptions, secondary rows', class: 'text-caption text-text-2' },
		{ token: 'nm-size-label', spec: '11px · Geist Mono 400', sample: '402 insufficient_balance · req_9d41e0b2', use: 'Machine output, meta, kbd', class: 'font-mono text-label text-text-2' },
		{ token: 'nm-size-label', spec: '11px · Geist Mono 400', sample: "What's in this project", use: 'Section label (only uppercase style)', class: 'font-mono text-label tracking-label text-text-3 uppercase' },
	];
</script>

<GallerySection
	title="Type"
	description="Two families, five sizes, three weights (400 / 500 / 600). Max four sizes per screen. Sentence case everywhere; uppercase only for the mono label style."
>
	<div class="flex flex-col">
		{#each styles as style, index (index)}
			<div class="flex items-baseline gap-6 border-b border-border py-4">
				<div class="flex w-44 shrink-0 flex-col gap-1">
					<span class="font-mono text-label text-text">{style.token}</span>
					<span class="font-mono text-label text-text-3">{style.spec}</span>
				</div>
				<p class="min-w-0 flex-1 {style.class}">{style.sample}</p>
				<p class="w-56 shrink-0 text-caption text-text-3">{style.use}</p>
			</div>
		{/each}
	</div>
</GallerySection>
