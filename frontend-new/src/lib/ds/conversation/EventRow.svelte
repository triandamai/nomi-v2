<script lang="ts">
	import { Rocket, RotateCcw, Globe, Database } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Something that happened to a project, logged in mono with its time
	// ("deploy #14 · main@a3f9c21 · built in 41s · 15:38").
	type Props = { kind: 'deploy' | 'rollback' | 'domain' | 'database'; text: string; time: string };
	let { kind, text, time }: Props = $props();

	const icons = { deploy: Rocket, rollback: RotateCcw, domain: Globe, database: Database };
</script>

<div class="flex w-full items-center gap-2 rounded-sm bg-surface px-3 py-2">
	<span class="inline-flex text-text-2"><Icon icon={icons[kind]} size={14} /></span>
	<span class="min-w-0 flex-1 truncate font-mono text-label text-text-2">{text}</span>
	<span class="font-mono text-label text-text-3">{time}</span>
</div>
