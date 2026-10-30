<script lang="ts">
	import { ArrowBigUp, Delete, Mic, Smile } from '@lucide/svelte';

	// The phone's own keyboard under the composer, as the design draws it: decoration, not the
	// app's. Its colours are the pen's kb tokens; the key shadow is the keyboard's own.
	const rows = ['qwertyuiop', 'asdfghjkl', 'zxcvbnm'];
</script>

<div aria-hidden="true" class="flex w-97.5 max-w-full flex-col gap-3 bg-kb-bg px-0.75 pt-1.5 pb-1 text-text">
	<div class="flex h-9 items-center">
		{#each ['ya', 'oke', 'nanti'] as word, index (word)}
			<span class={['flex h-9 flex-1 items-center justify-center text-phone', index < 2 && 'border-r border-kb-key-alt']}>{word}</span>
		{/each}
	</div>
	<div class="flex gap-1.5 px-0.75">
		{#each rows[0] as key (key)}<span class="key flex h-11 flex-1 items-center justify-center rounded-sm bg-kb-key">{key}</span>{/each}
	</div>
	<div class="flex gap-1.5 px-5.5">
		{#each rows[1] as key (key)}<span class="key flex h-11 flex-1 items-center justify-center rounded-sm bg-kb-key">{key}</span>{/each}
	</div>
	<div class="flex gap-1.5 px-0.75">
		<span class="key flex size-11 shrink-0 items-center justify-center rounded-sm bg-kb-key-alt"><ArrowBigUp size={20} strokeWidth={1.5} /></span>
		<span class="w-2 shrink-0"></span>
		{#each rows[2] as key (key)}<span class="key flex h-11 flex-1 items-center justify-center rounded-sm bg-kb-key">{key}</span>{/each}
		<span class="w-2 shrink-0"></span>
		<span class="key flex size-11 shrink-0 items-center justify-center rounded-sm bg-kb-key-alt"><Delete size={20} strokeWidth={1.5} /></span>
	</div>
	<div class="flex gap-1.5 px-0.75">
		<span class="key flex h-11 w-23 shrink-0 items-center justify-center rounded-sm bg-kb-key-alt text-body">123</span>
		<span class="key flex h-11 flex-1 items-center justify-center rounded-sm bg-kb-key text-body">space</span>
		<span class="key flex h-11 w-23 shrink-0 items-center justify-center rounded-sm bg-kb-key-alt text-body">return</span>
	</div>
	<div class="flex h-11 items-center justify-between px-5.5 text-text-2">
		<Smile size={26} strokeWidth={1.5} /><Mic size={24} strokeWidth={1.5} />
	</div>
</div>

<style>
	/* The keyboard's own keys: 22px letters and a 1px drop, as the pen draws the phone's keyboard. */
	.key {
		font-size: 22px;
		line-height: 28px;
		box-shadow: 0 1px 0 rgb(0 0 0 / 0.35);
	}
	.key.text-body {
		font-size: 15px;
		line-height: 19px;
	}
</style>
