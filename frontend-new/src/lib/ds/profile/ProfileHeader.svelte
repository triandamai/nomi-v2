<script lang="ts">
	import ProfileAvatar from './ProfileAvatar.svelte';

	// App / Settings / Profile (U9Gnrg) → Identity: the picture, the name as the page's title, and
	// the mono line under it ("@mario · Jakarta · joined 14 Jul 2026 · Free plan"), 16 apart.
	type Props = { name: string; line: string; avatar?: string };
	let { name, line, avatar }: Props = $props();
</script>

<header class="flex items-center gap-4">
	<ProfileAvatar {name} src={avatar} />
	<div class="flex min-w-0 flex-col gap-1">
		<h1 class="truncate text-display font-medium tracking-display text-text">{name}</h1>
		<p class="truncate font-mono text-label text-text-3">{line}</p>
	</div>
</header>
