<script lang="ts">
	import Button from '../actions/Button.svelte';

	// App / Settings / Profile (U9Gnrg) → Save Row: how many changes aren't saved yet, in the warning
	// colour and mono, then Discard and Save (Inverse, the panel's one action), 8 apart. With nothing
	// changed it says nothing and both wait.
	type Props = { pending: string; dirty: boolean; saving?: boolean; discard: string; save: string; ondiscard?: () => void; onsave?: () => void };
	let { pending, dirty, saving = false, discard, save, ondiscard, onsave }: Props = $props();
</script>

<div class="flex items-center gap-2">
	<span role="status" class="min-w-0 flex-1 truncate font-mono text-label text-warning">{dirty ? pending : ''}</span>
	<Button label={discard} variant="ghost" disabled={!dirty || saving} onclick={ondiscard} />
	<Button label={save} variant="inverse" type="submit" disabled={!dirty} loading={saving} onclick={onsave} />
</div>
