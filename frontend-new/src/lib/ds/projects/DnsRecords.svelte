<script lang="ts">
	import { Check, Copy } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import { createCopier } from '../actions/copier.svelte';

	// Flow / Build — Workspace tools (U16wU) → Screen / Domain (J2hPJ) and Domain connected (Dd60u)
	// → DNS: the records to set at the registrar for a domain of the person's own. Its heading in
	// the mono label with when it was last checked on the right, 8 above a table outlined in border
	// (radius 6): the header row on surface-2 ([10,12], 12 apart), then a row per record under a 1px
	// line, each column as drawn (type 56, name 110, the value filling with its copy button, status
	// 96), all mono at 11: found in the accent, waiting in the warning colour.
	type DnsRecord = { type: string; name: string; value: string; found: boolean };
	type Props = {
		records: DnsRecord[];
		/** "last checked 08:51 · every 5 min". */
		checked?: string;
		labels: { heading: string; type: string; name: string; value: string; status: string; found: string; waiting: string; copy: string };
	};
	let { records, checked, labels }: Props = $props();

	const id = $props.id();
	const copier = createCopier();
	let copied = $state<string | null>(null);
	function copy(value: string) {
		copied = value;
		copier.copy(value);
	}
</script>

<div class="flex flex-col gap-2">
	<p class="flex items-center justify-between gap-3 font-mono text-label text-text-3">
		<span id="{id}-heading" class="tracking-label uppercase">{labels.heading}</span>
		{#if checked}<span class="truncate">{checked}</span>{/if}
	</p>
	<div data-part="records" class="overflow-hidden rounded-sm outline -outline-offset-1 outline-border">
		<table aria-labelledby="{id}-heading" class="w-full border-collapse font-mono text-label">
			<thead>
				<tr class="bg-surface-2 text-left text-text-3">
					<th scope="col" class="py-2.5 pr-3 pl-3 font-normal"><span class="block w-14 uppercase">{labels.type}</span></th>
					<th scope="col" class="py-2.5 pr-3 font-normal"><span class="block w-27.5 uppercase">{labels.name}</span></th>
					<th scope="col" class="py-2.5 pr-3 font-normal uppercase">{labels.value}</th>
					<th scope="col" class="py-2.5 pr-3 font-normal"><span class="block w-24 uppercase">{labels.status}</span></th>
				</tr>
			</thead>
			<tbody>
				{#each records as record (`${record.type} ${record.name}`)}
					<tr class="border-t border-border">
						<td class="pt-2.25 pr-3 pb-2.5 pl-3 align-middle text-text"><span class="block w-14">{record.type}</span></td>
						<td class="pt-2.25 pr-3 pb-2.5 align-middle text-text"><span class="block w-27.5 truncate">{record.name}</span></td>
						<td class="pt-2.25 pr-3 pb-2.5 align-middle">
							<span class="flex min-w-0 items-center gap-3">
								<span class="min-w-0 flex-1 break-all text-text-2">{record.value}</span>
								<button
									type="button"
									aria-label="{labels.copy} {record.value}"
									class="inline-flex shrink-0 cursor-pointer rounded-xs text-text-3 outline-offset-1 hover:text-text pointer-coarse:min-h-11 pointer-coarse:min-w-11 pointer-coarse:items-center pointer-coarse:justify-center"
									onclick={() => copy(record.value)}
								>
									<Icon icon={copier.copied && copied === record.value ? Check : Copy} size={13} />
								</button>
							</span>
						</td>
						<td class="pt-2.25 pr-3 pb-2.5 align-middle"><span class={['block w-24', record.found ? 'text-accent' : 'text-warning']}>{record.found ? labels.found : labels.waiting}</span></td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
</div>
