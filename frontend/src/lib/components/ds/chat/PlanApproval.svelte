<script lang="ts">
	import { Play, Send } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import RunOptions from './RunOptions.svelte';

	// Chat responses → Plan Approval: how much of the plan may run on its own, the recommendation
	// picked first, or keep planning with what you want changed. The button says what will happen.
	type Option = { title: string; detail?: string; recommended?: boolean };
	type Props = { options: Option[]; who?: string; onrun: (choice: { choice: number } | { keepPlanning: string }) => void };
	let { options, who = 'Nomi', onrun }: Props = $props();

	const id = $props.id();
	let value = $state(-1);
	let changes = $state('');

	const chosen = $derived(value >= 0 ? value : Math.max(0, options.findIndex((option) => option.recommended)));
	const keep = $derived(chosen === options.length);

	function run() {
		onrun(keep ? { keepPlanning: changes.trim() } : { choice: chosen });
	}

	function onkeydown(event: KeyboardEvent) {
		const target = event.target as HTMLElement;
		const typing = target instanceof HTMLInputElement && target.type === 'text';
		if (event.key === 'Enter' && !(target instanceof HTMLButtonElement)) {
			event.preventDefault();
			run();
		} else if (!typing && /^[1-9]$/.test(event.key) && Number(event.key) <= options.length + 1) {
			event.preventDefault();
			value = Number(event.key) - 1;
		}
	}
</script>

<!-- Keys work anywhere inside the card; each control is also reachable on its own. -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<section class="flex w-full flex-col overflow-hidden rounded-md bg-bg outline -outline-offset-1 outline-border-strong" aria-labelledby="{id}-title" {onkeydown}>
	<header class="flex items-center gap-2.5 border-b border-border bg-surface px-3.5 pt-2.5 pb-2.25">
		<span class="inline-flex text-text-2"><Icon icon={Play} size={14} /></span>
		<p id="{id}-title" class="min-w-0 flex-1 text-ui font-medium text-text">{m.plan_approval_title()}</p>
	</header>
	<div class="p-3.5">
		<RunOptions name="{id}-mode" label={m.plan_approval_title()} {options} {who} bind:value bind:changes />
	</div>
	<footer class="flex flex-wrap items-center gap-2 border-t border-border px-3.5 pt-2.25 pb-2.5">
		<span class="min-w-0 flex-1 basis-40 font-mono text-label text-text-3">{m.plan_approval_keys({ count: String(options.length + 1) })}</span>
		<span class="ml-auto"><Button label={keep ? m.plan_approval_send() : m.plan_approval_run()} iconEnd={keep ? Send : Play} disabled={keep && !changes.trim()} onclick={run} /></span>
	</footer>
</section>
