<script lang="ts">
	import { Folder, MessageSquare } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import { rowStyle } from './rowStyle';

	// Design System / Nomi → Navigation: a recent chat or project, with how long ago in mono.
	type Props = { href: string; kind: 'chat' | 'project'; title: string; when: string; active?: boolean };
	let { href, kind, title, when, active = false }: Props = $props();

	const style = rowStyle();
</script>

<a
	{href}
	aria-current={active ? 'page' : undefined}
	class={[
		'flex h-7.5 w-full items-center gap-2 rounded-sm text-ui outline-offset-1 pointer-coarse:min-h-11',
		style.dense ? 'px-2.5' : 'px-3',
		active ? 'bg-surface-2 text-text' : 'text-text-2 hover:bg-surface-2 hover:text-text',
	]}
>
	<span class={['inline-flex', style.icon === 'text-2' ? 'text-text-2' : 'text-text-3']}><Icon icon={kind === 'chat' ? MessageSquare : Folder} size={14} /></span>
	<span class="min-w-0 flex-1 truncate">{title}</span>
	<span class="font-mono text-label text-text-3">{when}</span>
</a>
