<script lang="ts">
	import { Cpu, Sparkles } from '@lucide/svelte';
	import RegenerateMenu from './RegenerateMenu.svelte';
	import Button from '../actions/Button.svelte';

	// A RegenerateMenu off a button, for its test and the gallery.
	let { onpick }: { onpick: (choice: unknown) => void } = $props();
</script>

<RegenerateMenu
	models={[
		{ name: 'claude-sonnet', source: 'your key', icon: Cpu },
		{ name: 'deepseek-flash', source: 'Nomi credits', icon: Sparkles },
	]}
	{onpick}
>
	{#snippet trigger(props)}<Button label="Regenerate" variant="secondary" {...props} />{/snippet}
</RegenerateMenu>
