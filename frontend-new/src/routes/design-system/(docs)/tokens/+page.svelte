<script lang="ts">
	import Search from '$lib/ds/inputs/Search.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import exported from '$lib/ds/tokens/tokens.json';
	import theme from '../../../../app.css?raw';
	import { tokenGroups } from '$lib/docs/tokens';

	// Every token as the pen exports it, with the class that reaches it and both themes' values.
	const groups = tokenGroups(exported.tokens, theme);
	const count = groups.reduce((sum, group) => sum + group.rows.length, 0);
	let filter = $state('');
	const matches = (text: string) => filter.toLowerCase().split(/\s+/).every((word) => text.toLowerCase().includes(word));
</script>

<DocPage
	eyebrow="Resources"
	title="Tokens"
	lede="Every colour, size, space and radius in Nomi is a token: named once in the pen, exported to tokens.json, and the only values the code can use. Use the class; the class follows the theme."
	wide
>
	{#snippet meta()}
		<span class="font-mono text-label text-text-3">{count} tokens · exported from {exported.source} on {exported.exported}</span>
	{/snippet}

	<Note>
		<p>
			Write <code class="font-mono text-caption text-text">bg-surface</code>, never a hex value: only the design's tokens exist in the code, and a test fails on raw colours,
			<code class="font-mono text-caption text-text">dark:</code>
			and arbitrary values. When the pen changes, re-export <code class="font-mono text-caption text-text">tokens.json</code>; a test fails until
			<code class="font-mono text-caption text-text">tokens.css</code> matches it.
		</p>
	</Note>

	<div class="max-w-90"><Search placeholder="Filter tokens" label="Filter tokens" bind:value={filter} /></div>

	{#each groups as group (group.title)}
		{@const rows = group.rows.filter((row) => matches(`${row.name} ${row.classes.join(' ')} ${row.light ?? ''} ${row.dark ?? ''} ${row.value ?? ''}`))}
		{#if rows.length}
			<DocSection title={group.title}>
				<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
					<table class="w-full min-w-170 border-collapse text-left">
						<thead>
							<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
								<th scope="col" class="px-4 py-2.5 font-normal">Token</th>
								<th scope="col" class="px-4 py-2.5 font-normal">Classes</th>
								{#if rows[0].dark}
									<th scope="col" class="w-44 px-4 py-2.5 font-normal">Dark</th>
									<th scope="col" class="w-44 px-4 py-2.5 font-normal">Light</th>
								{:else}
									<th scope="col" class="w-44 px-4 py-2.5 font-normal">Value</th>
								{/if}
							</tr>
						</thead>
						<tbody>
							{#each rows as row (row.name)}
								<tr class="border-b border-border last:border-b-0">
									<th scope="row" class="px-4 py-2.5 text-left font-normal"><code class="font-mono text-caption text-text">{row.cssVar}</code></th>
									<td class="px-4 py-2.5 font-mono text-caption text-text-2">{row.classes.join(' · ') || '—'}</td>
									{#if row.dark}
										{#each [['dark', row.dark], ['light', row.light]] as [mode, value] (mode)}
											<td class="px-4 py-2.5">
												<span class="flex items-center gap-2.5">
													<span data-theme={mode} class="size-5 shrink-0 rounded-xs outline -outline-offset-1 outline-border-strong" style:background="var({row.cssVar})"></span>
													<code class="font-mono text-caption text-text-2">{value}</code>
												</span>
											</td>
										{/each}
									{:else}
										<td class="px-4 py-2.5 font-mono text-caption text-text-2">{row.value}</td>
									{/if}
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			</DocSection>
		{/if}
	{/each}
</DocPage>
