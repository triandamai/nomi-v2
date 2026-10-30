<script lang="ts">
	import { Settings } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Avatar from '../identity/Avatar.svelte';

	/** The person, at the foot of the sidebar, with the way into settings. */
	let { name, avatar, settingsHref }: { name: string; avatar?: string; settingsHref: string } = $props();
</script>

<div class="flex items-center gap-2.5 px-2 py-1">
	<Avatar {name} src={avatar} />
	<span class="min-w-0 flex-1 truncate text-ui text-text">{name}</span>
	<a href={settingsHref} aria-label={m.nav_settings()} class="inline-flex items-center justify-center rounded-xs text-text outline-offset-2 hover:text-text-2 pointer-coarse:size-11">
		<Icon icon={Settings} />
	</a>
</div>
