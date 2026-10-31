<script lang="ts">
	import type { ComponentApi } from '../api/extract';

	// A component's props, straight from the type it gives $props(): read at build time, so it is
	// always the code's.
	let { api }: { api: ComponentApi } = $props();
	const types = $derived(Object.entries(api.types));
</script>

{#if api.props.length}
	<div class="overflow-x-auto rounded-md outline -outline-offset-1 outline-border">
		<table class="w-full min-w-170 border-collapse text-left">
			<thead>
				<tr class="border-b border-border bg-surface font-mono text-label tracking-label text-text-3 uppercase">
					<th scope="col" class="w-40 px-4 py-2.5 font-normal">Prop</th>
					<th scope="col" class="px-4 py-2.5 font-normal">Type</th>
					<th scope="col" class="w-36 px-4 py-2.5 font-normal">Default</th>
				</tr>
			</thead>
			<tbody>
				{#each api.props as prop (prop.name)}
					<tr class="border-b border-border align-top last:border-b-0">
						<th scope="row" class="px-4 py-3 text-left font-normal">
							<span class="flex flex-col items-start gap-1">
								<code class="font-mono text-caption text-text">{prop.name}</code>
								{#if prop.required}<span class="font-mono text-micro text-accent">required</span>{/if}
								{#if prop.bindable}<span class="font-mono text-micro text-text-3">bindable</span>{/if}
							</span>
						</th>
						<td class="px-4 py-3">
							<code class="font-mono text-caption break-words text-text-2">{prop.type}</code>
							{#if prop.doc}<p class="mt-1.5 text-caption text-text-3">{prop.doc}</p>{/if}
						</td>
						<td class="px-4 py-3"><code class="font-mono text-caption text-text-2">{prop.default ?? '—'}</code></td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{:else}
	<p class="text-ui text-text-3">No props: it draws the same thing every time.</p>
{/if}
{#if api.attributes.length || api.rest}
	<p class="text-caption text-text-3">
		{#if api.attributes.length}Also takes {api.attributes.map((family) => `${family} attributes`).join(' and ')}.{/if}
		{#if api.rest}Other attributes go on to its element.{/if}
	</p>
{/if}
{#if types.length}
	<dl class="flex flex-col gap-2">
		{#each types as [name, type] (name)}
			<div class="flex flex-wrap gap-x-2 font-mono text-caption">
				<dt class="text-code-type">type {name} =</dt>
				<dd class="m-0 break-words text-text-2">{type}</dd>
			</div>
		{/each}
	</dl>
{/if}
