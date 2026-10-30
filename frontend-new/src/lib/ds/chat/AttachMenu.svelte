<script lang="ts">
	import type { ComponentProps } from 'svelte';
	import { HardDrive, Image, Link2, Paperclip } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Menu from '../overlays/Menu.svelte';

	// Chat responses → Attach Menu, off the composer's paperclip: upload, a photo or screenshot,
	// a Drive file (through the crew member who has Drive), or a link; with the limits underneath.
	type Source = 'file' | 'photo' | 'drive' | 'link';
	type Props = {
		/** Who reaches Google Drive for you, e.g. "Tara"; without one, Drive isn't offered. */
		driveBy?: string;
		/** The largest file a chat takes, e.g. "20 MB". */
		max: string;
		onpick: (source: Source) => void;
		trigger: ComponentProps<typeof Menu>['trigger'];
	};
	let { driveBy, max, onpick, trigger }: Props = $props();
</script>

<Menu
	wide
	{trigger}
	footer={m.attach_limits({ max })}
	items={[
		{ label: m.attach_file(), icon: Paperclip, onselect: () => onpick('file') },
		{ label: m.attach_photo(), icon: Image, onselect: () => onpick('photo') },
		...(driveBy ? [{ label: m.attach_drive(), icon: HardDrive, meta: driveBy, onselect: () => onpick('drive') }] : []),
		{ label: m.attach_link(), icon: Link2, onselect: () => onpick('link') },
	]}
/>
