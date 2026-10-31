<script lang="ts">
	import { Mic } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Voice Transcribed: under the composer, a note that its text came from audio
	// and can be edited before sending.
	let { duration }: { duration: string } = $props();
</script>

<p class="flex items-start gap-1.5 font-mono text-label text-text-3"><span class="inline-flex h-3.5 shrink-0 items-center"><Icon icon={Mic} size={12} /></span>{m.voice_transcribed({ duration })}</p>
