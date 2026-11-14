<script lang="ts">
	import TextField from '../inputs/TextField.svelte';
	import Checkbox from '../inputs/Checkbox.svelte';

	// Admin → Secret field: an API key, a password, a signing token. Write-only: it never holds a
	// stored secret (the API never sends one back), only whether one is set and where from, said
	// under the field ("Saved here · leave empty to keep it"). What the person types is sent once and
	// the field empties again. Browsers aren't offered to fill it with a sign-in password. With
	// `forget`, a checkbox removes the saved one instead.
	type Props = {
		label: string;
		name: string;
		/** Whether a secret is set now, saved or from the environment. */
		set: boolean;
		/** Where it comes from, and what leaving the field empty does. */
		status: string;
		placeholder?: string;
		/** The new secret being typed; starts empty. */
		value?: string;
		error?: string;
		disabled?: boolean;
		required?: boolean;
		/** Offers "forget the saved one": the checkbox's name and label. */
		forget?: { name: string; label: string };
	};
	let { label, name, set, status, placeholder, value = $bindable(''), error, disabled = false, required = false, forget }: Props = $props();
</script>

<div class="flex w-full flex-col gap-2" data-set={set || undefined}>
	<TextField {label} {name} type="password" mono autocomplete="new-password" bind:value {placeholder} helper={status} {error} {disabled} required={required && !set} />
	{#if forget && set}
		<Checkbox label={forget.label} name={forget.name} {disabled} />
	{/if}
</div>
