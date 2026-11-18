<script lang="ts">
	import { Play } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import BottomSheet from '../overlays/BottomSheet.svelte';
	import PhoneOption from '../mobile/PhoneOption.svelte';
	import PhoneButton from '../mobile/PhoneButton.svelte';

	// Flow / Mobile Chat — Clarify → plan, step 7 (KCocj): how a plan should run, in a sheet. "Run
	// this plan?" with its steps and who does them, the ways to run it as Mobile / Option rows 8
	// apart (the recommended one first, picked to start), then Not now (the bar stays) and Run plan.
	type Option = { title: string; detail?: string; recommended?: boolean };
	type Props = {
		open?: boolean;
		/** "5 steps · Nomi, Dana, Tara". */
		subtitle: string;
		options: Option[];
		onrun: (index: number) => void;
		onclose?: () => void;
	};
	let { open = $bindable(false), subtitle, options, onrun, onclose }: Props = $props();

	const id = $props.id();
	const first = () => Math.max(0, options.findIndex((option) => option.recommended));
	let chosen = $state(first());
	let group: HTMLElement | undefined = $state();
	$effect(() => {
		if (!open) return;
		chosen = first();
		requestAnimationFrame(() => group?.querySelector<HTMLInputElement>('input:checked')?.focus());
	});
</script>

<BottomSheet bind:open title={m.phone_run_title()} {subtitle} {onclose}>
	<div class="flex flex-col gap-2 px-4 pb-2">
		<div bind:this={group} role="radiogroup" aria-label={m.phone_run_title()} class="flex flex-col gap-2">
			{#each options as option, index (index)}
				<PhoneOption name="{id}-run" key={index + 1} title={option.title} detail={option.detail} recommended={option.recommended} checked={chosen === index} onselect={() => (chosen = index)} />
			{/each}
		</div>
		<div class="flex gap-2 pt-2">
			<PhoneButton variant="secondary" label={m.phone_run_not_now()} onclick={() => (open = false)} />
			<PhoneButton label={m.phone_run_run()} icon={Play} onclick={() => onrun(chosen)} />
		</div>
	</div>
</BottomSheet>
