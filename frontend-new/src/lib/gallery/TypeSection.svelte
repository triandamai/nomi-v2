<script lang="ts">
	import GallerySection from './GallerySection.svelte';

	const styles = [
		{ token: 'nm-size-display', spec: '28px · Geist 500', sample: 'Morning, Mario.', use: 'Page titles, greeting' },
		{ token: 'nm-size-body', spec: '15px · Geist 400', sample: "I couldn't reply because your DeepSeek account is out of credits.", use: 'Message body, page intro' },
		{ token: 'nm-size-ui', spec: '13px · Geist 500', sample: 'Open workspace', use: 'Buttons, nav, rows, labels' },
		{ token: 'nm-size-caption', spec: '12px · Geist 400', sample: 'Transactions, budgets, subscriptions', use: 'Descriptions, secondary rows' },
		{ token: 'nm-size-label', spec: '11px · Geist Mono 400', sample: '402 insufficient_balance · req_9d41e0b2', use: 'Machine output, meta, kbd' },
		{ token: 'nm-size-label', spec: '11px · Geist Mono 400', sample: "What's in this project", use: 'Section label (only uppercase style)' },
	];
</script>

<GallerySection
	title="Type"
	description="Two families, five sizes, three weights (400 / 500 / 600). Max four sizes per screen. Sentence case everywhere; uppercase only for the mono label style."
>
	<!-- The pen's Type Table: one bordered, rounded table, a row per style. -->
	<div data-type-table class="flex flex-col overflow-hidden rounded-md outline -outline-offset-1 outline-border">
		{#each styles as style, index (index)}
			<div class="flex flex-col gap-2 border-b border-border px-5 pt-4 pb-3.75 last:border-b-0 last:pb-4 md:flex-row md:items-center md:gap-6">
				<div class="flex shrink-0 flex-col gap-0.5 md:w-50">
					<span class="font-mono text-label text-text">{style.token}</span>
					<span class="font-mono text-label text-text-3">{style.spec}</span>
				</div>
				<p
					class={[
						'min-w-0 flex-1 break-words',
						index === 0 && 'text-display font-medium tracking-display text-text',
						index === 1 && 'text-body text-text',
						index === 2 && 'text-ui font-medium text-text',
						index === 3 && 'text-caption text-text',
						index === 4 && 'font-mono text-label text-text',
						index === 5 && 'font-mono text-label tracking-label text-text-3 uppercase',
					]}
				>
					{style.sample}
				</p>
				<p class="shrink-0 text-caption text-text-3 md:w-55">{style.use}</p>
			</div>
		{/each}
	</div>
</GallerySection>
