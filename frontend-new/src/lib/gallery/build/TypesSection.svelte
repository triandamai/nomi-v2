<script lang="ts">
	import GallerySection from '../GallerySection.svelte';

	const columns = ['Type', 'Koda sets up', 'Workspace tabs', 'Share leads with', 'Monitor leads with'];
	const types: [type: string, setUp: string, tabs: string, share: string, monitor: string][] = [
		['Landing page', 'Pages, contact form, search preview', 'Deployments · Domain · Monitor · Settings', 'Link + preview card', 'Visits · sources · form sends'],
		['Web app with data', 'Pages, database, admin view', '+ Database', 'Link · QR for a counter', 'Requests · orders · errors'],
		['Catalog / shop', 'Catalog, cart, WhatsApp order hand-off', '+ Database', 'Link · product preview', 'Orders · top items'],
		['Internal tool', 'Forms, tables, sign-in for the team', '+ Database · People first', 'Unlisted link for the team', 'Active users · errors'],
		['Portfolio / blog', 'Pages, posts, contact', 'Deployments · Domain · Monitor · Settings', 'Link + preview card', 'Visits · top posts'],
	];
</script>

<GallerySection
	title="Build types"
	description="Same workspace for every build. Only what the app actually has shows up: no database tab for a landing page, no QR unless it's for a physical place."
>
	<div class="flex flex-col gap-6">
		<div role="table" aria-label="Build types" class="flex flex-col overflow-hidden rounded-md border border-border bg-bg">
			<div role="row" class="flex gap-4 border-b border-border bg-surface px-3.75 py-2.75">
				{#each columns as column, index (column)}
					<span
						role="columnheader"
						class={[
							'font-mono text-label leading-comfy tracking-label text-text-3 uppercase',
							index === 0 ? 'w-50 shrink-0' : index < 3 ? 'min-w-0 flex-1' : 'w-55 shrink-0',
						]}>{column}</span
					>
				{/each}
			</div>
			{#each types as [type, ...cells], index (type)}
				<div role="row" class={['flex gap-4 border-border px-3.75', index === 0 ? 'pt-3' : 'border-t pt-2.75', index === types.length - 1 ? 'pb-2.75' : 'pb-3']}>
					<span role="rowheader" class="w-50 shrink-0 text-ui leading-comfy font-medium text-text">{type}</span>
					{#each cells as cell, column (column)}
						<span role="cell" class={['text-caption leading-comfy text-text-2', column < 2 ? 'min-w-0 flex-1' : 'w-55 shrink-0']}>{cell}</span>
					{/each}
				</div>
			{/each}
		</div>
		<p class="text-caption leading-normal text-text-3">
			Rules: the Database tab and dump only exist when the app stores data. QR, embed and print live under More ways to share for every
			type. Estimate “You get” names exactly what will exist.
		</p>
	</div>
</GallerySection>
