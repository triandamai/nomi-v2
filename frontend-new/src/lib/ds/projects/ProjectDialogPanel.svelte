<script lang="ts">
	import type { Snippet } from 'svelte';

	// Design System / Projects → the dialogs' panel (Stop m7QPG7, Deploy A3CDJ, Move dEQNS, Invite
	// RtsT2, GitHub Connect s3mLi, Repo Picker tzhsm): the Nomi sheet's Dialog box (surface,
	// border-strong inside, the float shadow; title and description 8 apart in a 20 body, actions in
	// a 54 footer under a line drawn inside), at the widths the projects pen draws (420, 440, 460,
	// 520) and Flow / Build's Publish (480). overlays/DialogPanel keeps the old app's twin's props,
	// so the widths live here.
	type Props = { id: string; title: string; description?: string; width?: 420 | 440 | 460 | 480 | 520; children?: Snippet; actions?: Snippet };
	let { id, title, description, width = 420, children, actions }: Props = $props();
</script>

<div
	data-testid="dialog-panel"
	class={[
		'flex max-w-full flex-col overflow-hidden rounded-md bg-surface text-text shadow-float outline -outline-offset-1 outline-border-strong',
		width === 420 && 'w-105',
		width === 440 && 'w-110',
		width === 460 && 'w-115',
		width === 480 && 'w-120',
		width === 520 && 'w-130',
	]}
>
	<div class="flex flex-col gap-2 p-5">
		<h2 id="{id}-title" class="text-body font-medium text-text">{title}</h2>
		{#if description}<p id="{id}-description" class="text-ui leading-normal text-text-2">{description}</p>{/if}
		{@render children?.()}
	</div>
	{#if actions}
		<div class="flex flex-wrap items-center justify-end gap-2 border-t border-border px-5 pt-2.75 pb-3">{@render actions()}</div>
	{/if}
</div>
