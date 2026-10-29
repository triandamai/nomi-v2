<script lang="ts">
	import GallerySection from '../GallerySection.svelte';

	const story: [key: string, value: string][] = [
		['User', 'Mario Prasetyo'],
		['Project', 'warung-kopi-menu · "Menu and pre-order page for Warung Kopi Bu Siti"'],
		['Code verified', '08:11'],
		['Run pressed / build started', '08:13'],
		['Building frame', 'writing src/routes/menu/+page.svelte · 01:12 · step 2 of 6 · 5 of 9 files'],
		['Live', 'deploy #1 · 08:18 · 05:40 elapsed · main@4be1c07 · 9 files'],
		['URL', 'warung-kopi-menu-3f9a12.trian.space  ([hosting domain] placeholder)'],
		['Estimate', '1 build · 4–7 min · 38K–62K tokens · Rp (38–62) × [cost per 1K tokens] · Free allowance'],
		['Actual', '1 build · 05:40 · 51.2K tokens · Rp 51.2 × [cost per 1K tokens] · Free allowance'],
		['Clarify answers', 'Pay at pickup · Kopi + Makanan · pickup 07:00–11:00, 15-min slots'],
		['Other project', 'salah-daily-tracker · deploy #14 (everywhere)'],
		[
			'Open values',
			'[Pro price IDR] [Free live-project cap] [Free builds/month] [Pro allowance] [hosting domain] [e-wallet list] [payment provider] [cost per 1K tokens]',
		],
	];
</script>

<GallerySection title="Story sheet" description="Every frame in Flow / Build uses these values. Brackets are not decided yet and stay visible in the design.">
	<dl class="flex flex-col rounded-md border border-border">
		{#each story as [key, value] (key)}
			<div class="flex gap-6 border-border px-4.75 pt-2.75 pb-3 not-first:border-t last:pb-2.75">
				<dt class="w-55 shrink-0 text-ui text-text-2">{key}</dt>
				<dd class="min-w-0 flex-1 font-mono text-ui whitespace-pre-wrap text-text">{value}</dd>
			</div>
		{/each}
	</dl>
</GallerySection>
