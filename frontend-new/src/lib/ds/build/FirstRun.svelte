<script lang="ts">
	import { ArrowRight } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '../actions/Button.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import TextField from '../inputs/TextField.svelte';

	// Build → First run: the one screen after the sign-in code. A name (prefilled when we know it)
	// and one line on what Nomi does; no crew tour, the composer comes next.
	type Props = { name?: string; oncontinue: (name: string) => void };
	let { name = '', oncontinue }: Props = $props();

	const id = $props.id();
	const initial = () => name;
	let value = $state(initial());
</script>

<form
	aria-labelledby="{id}-title"
	class="flex w-110 max-w-full flex-col gap-6 rounded-md bg-surface p-8 outline -outline-offset-1 outline-border"
	onsubmit={(event) => {
		event.preventDefault();
		if (value.trim()) oncontinue(value.trim());
	}}
>
	<Mascot agent="nomi" size={32} />
	<div class="flex flex-col gap-2">
		<h1 id="{id}-title" class="text-display font-medium text-text">{m.firstrun_title()}</h1>
		<p class="text-body leading-normal text-text-2">{m.firstrun_line()}</p>
	</div>
	<TextField label={m.firstrun_name()} helper={m.firstrun_name_help()} smallHelper mono autocomplete="name" bind:value />
	<Button label={m.firstrun_continue()} type="submit" iconEnd={ArrowRight} disabled={!value.trim()} />
</form>
