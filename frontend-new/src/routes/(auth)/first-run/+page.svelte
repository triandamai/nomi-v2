<script lang="ts">
	// First run (jPO7K; phone FWXo1), with the design system's FirstRun as it is: the name, then on
	// to where the account was going. It sits in the sign-in layout's form column, as jPO7K draws
	// it; FirstRun brings its own card and mascot, so the layout's greeting is left out.
	import { applyAction, deserialize } from '$app/forms';
	import { page } from '$app/state';
	import FirstRun from '$lib/ds/build/FirstRun.svelte';
	import ErrorCallout from '$lib/ds/conversation/ErrorCallout.svelte';
	import { alertCode, sayAlert, sayField } from '$lib/auth/say';
	import { m } from '$lib/paraglide/messages';

	let { data, form } = $props();

	async function save(name: string) {
		const body = new FormData();
		body.set('name', name);
		const response = await fetch(`${page.url.pathname}${page.url.search}`, { method: 'POST', body, headers: { 'x-sveltekit-action': 'true' } });
		await applyAction(deserialize(await response.text()));
	}
</script>

<svelte:head><title>{m.firstrun_page_title()}</title></svelte:head>

<FirstRun name={form?.name || data.name} oncontinue={save} />
{#if form?.alert}<ErrorCallout title={sayAlert(form.alert)} code={alertCode(form.alert)} />{/if}
{#if form?.fields.name}<p class="text-caption text-danger" role="alert">{sayField(form.fields.name)}</p>{/if}
