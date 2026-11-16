<script lang="ts">
	import { untrack } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import SoulEditor from '$lib/components/tools/SoulEditor.svelte';
	import ToolPicker from '$lib/components/tools/ToolPicker.svelte';
	import { NAME_MAX_CHARS, ROLE_MAX_CHARS, SOUL_MAX_CHARS, TOOLS_SET_FIELD } from '$lib/crewSettings';
	import { addTool } from '$lib/toolPicker';
	import type { CrewTemplate } from '$lib/types';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	// The form's fields live here so a template can fill them (the person then adjusts and adds it);
	// after a failed save the server hands back what was typed.
	let name = $state(untrack(() => form?.name ?? ''));
	let role = $state(untrack(() => form?.role ?? ''));
	let soul = $state(untrack(() => form?.soul ?? ''));
	let template = $state(untrack(() => form?.template ?? ''));
	let tools = $state<string[]>(untrack(() => [...(form?.tools ?? [])]));
	let blank: HTMLElement | undefined = $state();

	function useTemplate(chosen: CrewTemplate) {
		name = chosen.name;
		role = chosen.role;
		soul = chosen.soul;
		template = chosen.key;
		tools = [...chosen.tools];
		blank?.scrollIntoView({ block: 'start' });
	}

	const errorText = $derived.by(() => {
		if (form && 'invalid' in form) {
			return form.invalid === 'name' ? m.crewnew_invalid_name() : form.invalid === 'role' ? m.crewnew_invalid_role() : m.crewnew_invalid_soul();
		}
		if (form && 'failed' in form) return form.failed === 'taken' ? m.crewnew_taken() : m.crewnew_failed();
		return null;
	});
</script>

<div class="new-page">
	<div class="new-page__inner">
		<a class="new-page__back" href="/crew">← {m.crewmember_back()}</a>
		<header>
			<h1 class="md-headline-medium new-page__title">{m.crewnew_title()}</h1>
			<p class="new-page__lede">{m.crewnew_lede()}</p>
		</header>
		{#if errorText}<p class="new-page__error" role="alert">{errorText}</p>{/if}

		{#if data.templates.length > 0}
			<section aria-labelledby="templates-title">
				<h2 id="templates-title" class="new-page__section">{m.crewnew_templates()}</h2>
				<ul class="templates">
					{#each data.templates as item (item.key)}
						<li class="template">
							<AgentShape agent={item.key} size={40} face />
							<div class="template__text">
								<span class="template__name">{item.name}</span>
								<span class="template__role">{item.role}</span>
							</div>
							<Button variant="tonal" size="s" type="button" onclick={() => useTemplate(item)}>{m.crewnew_use_template()}</Button>
						</li>
					{/each}
				</ul>
			</section>
		{/if}

		<section class="blank" aria-labelledby="blank-title" bind:this={blank}>
			<h2 id="blank-title" class="new-page__section">{m.crewnew_blank()}</h2>
			<form method="POST" use:enhance class="blank__form">
				<input type="hidden" name="template" value={template} />
				<input type="hidden" name={TOOLS_SET_FIELD} value="1" />
				<TextField label={m.crewnew_name()} name="name" bind:value={name} maxlength={NAME_MAX_CHARS} required error={!!form && 'invalid' in form && form.invalid === 'name'} />
				<TextField label={m.crewnew_role()} name="role" bind:value={role} maxlength={ROLE_MAX_CHARS} required error={!!form && 'invalid' in form && form.invalid === 'role'} />
				<div class="blank__soul">
					<span>{m.crewnew_soul()}</span>
					<SoulEditor bind:value={soul} name="soul" label={m.crewnew_soul()} maxlength={SOUL_MAX_CHARS} crew={data.crew} tools={data.catalog} ontoolmention={(tool) => (tools = addTool(tools, tool))} />
				</div>
				<ToolPicker label={m.crewnew_tools()} options={data.catalog} bind:selected={tools} name="tools" />
				<div class="blank__actions">
					<Button variant="filled" size="s" type="submit">{m.crewnew_create()}</Button>
				</div>
			</form>
		</section>
	</div>
</div>

<style>
	.new-page {
		height: 100%;
		overflow-y: auto;
		padding: 24px clamp(16px, 4vw, 44px) 44px;
		box-sizing: border-box;
		background: var(--nomi-color-ground);
	}
	.new-page__inner {
		display: flex;
		flex-direction: column;
		gap: 18px;
		max-width: 760px;
		margin: 0 auto;
	}
	.new-page__back {
		align-self: flex-start;
		min-height: 44px;
		display: inline-flex;
		align-items: center;
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: none;
	}
	.new-page__back:hover,
	.new-page__back:focus-visible {
		color: var(--md-sys-color-primary);
	}
	.new-page__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.new-page__lede {
		margin: 6px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.new-page__error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.new-page__section {
		margin: 0 0 10px;
		font-size: 1.0625rem;
		font-weight: 650;
		color: var(--md-sys-color-on-surface);
	}
	.templates {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 0;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		list-style: none;
	}
	.template {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 12px;
		padding: 12px 16px;
	}
	.template + .template {
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.template__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 160px;
	}
	.template__name {
		font-weight: 650;
		color: var(--md-sys-color-on-surface);
	}
	.template__role {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.blank__form {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 18px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.blank__soul {
		display: flex;
		flex-direction: column;
		gap: 6px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.blank__actions {
		display: flex;
		justify-content: flex-end;
	}
</style>
