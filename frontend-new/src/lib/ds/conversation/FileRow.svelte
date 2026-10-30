<script lang="ts">
	import { Check, Circle, FileCode, Loader } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// A file an agent touched, with its change ("+214"), in mono. While a build writes files it
	// also says where each one is: written (a mint check), being written (text colour, "writing"),
	// or still to come (faded).
	type Props = { path: string; change?: string; state?: 'done' | 'writing' | 'queued' };
	let { path, change, state }: Props = $props();
</script>

<span class="flex w-full items-center gap-2 px-3 py-1.5">
	<span class={['inline-flex', state === 'done' ? 'text-accent' : state === 'writing' ? 'animate-spin text-text motion-reduce:animate-none' : 'text-text-3']}>
		<Icon icon={state === 'done' ? Check : state === 'writing' ? Loader : state === 'queued' ? Circle : FileCode} size={12} />
	</span>
	<span class={['min-w-0 flex-1 truncate font-mono text-label', state === 'writing' ? 'text-text' : 'text-text-2']}>{path}</span>
	{#if state === 'writing'}
		<span class="font-mono text-label text-text-2">{m.file_writing()}</span>
	{:else if change}
		<span class="font-mono text-label text-text-3">{change}</span>
	{/if}
</span>
