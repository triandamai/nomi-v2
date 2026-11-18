<script lang="ts" module>
	/** One field of a form the crew filled in from the chat (the backend's form view). */
	export type PhoneFormField = {
		key: string;
		label: string;
		type: 'text' | 'textarea' | 'number' | 'date' | 'time' | 'select' | 'multiselect' | 'checkbox';
		required: boolean;
		options: string[];
		placeholder?: string;
	};
	export type PhoneFormValues = Record<string, string | boolean>;
</script>

<script lang="ts">
	import { CircleAlert } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import BottomSheet from '../overlays/BottomSheet.svelte';
	import PhoneButton from '../mobile/PhoneButton.svelte';

	// Flow / Mobile Chat — Plan → approve → act, step 3 (MKUyQ): a form the crew prefilled from the
	// chat, opened in a sheet named after it ("New reminder · Kala · prefilled from chat"). Fields
	// 14 apart at the phone's 16 (so iOS doesn't zoom): text, date and time as the platform's own,
	// a few options as the pen's segment (Once / Weekly / Monthly / Yearly), more as a select. A
	// date that has passed says so under its field ("That date has passed.") and Save waits until
	// it's fixed, as it waits for every required field. Cancel and Save.
	type Props = {
		open?: boolean;
		title: string;
		subtitle?: string;
		fields: PhoneFormField[];
		values?: PhoneFormValues;
		/** The person's today, as `YYYY-MM-DD`: dates before it have passed. */
		today?: string;
		submitLabel?: string;
		onsubmit: (values: PhoneFormValues) => void;
		onclose?: () => void;
	};
	let { open = $bindable(false), title, subtitle, fields, values = $bindable({}), today, submitLabel, onsubmit, onclose }: Props = $props();

	const id = $props.id();
	const text = (key: string) => (typeof values[key] === 'string' ? (values[key] as string) : '');
	const past = (field: PhoneFormField) => field.type === 'date' && Boolean(today) && /^\d{4}-\d{2}-\d{2}$/.test(text(field.key)) && text(field.key) < today!;
	const missing = $derived(fields.some((field) => field.required && field.type !== 'checkbox' && !text(field.key).trim()));
	const blocked = $derived(missing || fields.some(past));
	const set = (key: string, value: string | boolean) => (values = { ...values, [key]: value });
	const SEGMENTS = 4;

	let body: HTMLElement | undefined = $state();
	$effect(() => {
		if (open) requestAnimationFrame(() => body?.querySelector<HTMLElement>('input, select, textarea, [role="radio"]')?.focus());
	});
</script>

<BottomSheet bind:open {title} {subtitle} {onclose}>
	<form
		bind:this={body}
		class="flex flex-col gap-3.5 px-4 pt-1 pb-2"
		onsubmit={(event) => {
			event.preventDefault();
			if (!blocked) onsubmit(values);
		}}
	>
		{#each fields as field (field.key)}
			{@const error = past(field) ? m.phone_form_past_date() : undefined}
			{#if field.type === 'checkbox'}
				<label class="flex min-h-11 cursor-pointer items-center gap-3 text-phone text-text">
					<input type="checkbox" checked={values[field.key] === true} onchange={(event) => set(field.key, event.currentTarget.checked)} class="size-5.5 accent-accent outline-offset-2" />
					{field.label}
				</label>
			{:else if field.type === 'select' && field.options.length > 1 && field.options.length <= SEGMENTS}
				<div class="flex flex-col gap-1.5">
					<span id="{id}-{field.key}" class="text-ui font-medium text-text">{field.label}</span>
					<div role="radiogroup" aria-labelledby="{id}-{field.key}" class="flex gap-0.75 rounded-md bg-surface-2 p-0.75">
						{#each field.options as option (option)}
							{@const checked = text(field.key) === option}
							<button
								type="button"
								role="radio"
								aria-checked={checked}
								class={['relative h-8.5 min-w-0 flex-1 cursor-pointer rounded-inner text-phone-sm outline-offset-1', checked ? 'bg-bg font-semibold text-text' : 'text-text-2']}
								onclick={() => set(field.key, option)}
							>
								<span aria-hidden="true" class="absolute -inset-y-1.25 inset-x-0"></span>
								{option}
							</button>
						{/each}
					</div>
				</div>
			{:else}
				<div class="flex flex-col gap-1.5">
					<label for="{id}-{field.key}" class="text-ui font-medium text-text">{field.label}</label>
					<div class={['flex h-11 w-full items-center gap-2 rounded-sm border bg-surface px-2.75 focus-within:shadow-focus', error ? 'border-danger' : 'border-border-strong focus-within:border-accent']}>
						{#if field.type === 'select' || field.type === 'multiselect'}
							<select
								id="{id}-{field.key}"
								required={field.required}
								value={text(field.key)}
								onchange={(event) => set(field.key, event.currentTarget.value)}
								class="min-w-0 flex-1 bg-transparent text-phone text-text outline-none"
							>
								<option value=""></option>
								{#each field.options as option (option)}<option value={option}>{option}</option>{/each}
							</select>
						{:else}
							<input
								id="{id}-{field.key}"
								type={field.type === 'textarea' ? 'text' : field.type}
								required={field.required}
								placeholder={field.placeholder}
								value={text(field.key)}
								oninput={(event) => set(field.key, event.currentTarget.value)}
								aria-invalid={error ? 'true' : undefined}
								aria-describedby={error ? `${id}-${field.key}-error` : undefined}
								class="min-w-0 flex-1 bg-transparent text-phone text-text caret-accent outline-none placeholder:text-text-3"
							/>
						{/if}
						{#if error}<span class="inline-flex text-danger"><Icon icon={CircleAlert} size={14} /></span>{/if}
					</div>
					{#if error}<p id="{id}-{field.key}-error" role="alert" class="text-caption text-danger">{error}</p>{/if}
				</div>
			{/if}
		{/each}
		<div class="flex gap-2 pt-1">
			<PhoneButton variant="secondary" label={m.action_cancel()} onclick={() => (open = false)} />
			<button
				type="submit"
				aria-disabled={blocked}
				class={['flex h-11.5 w-full cursor-pointer items-center justify-center rounded-lg bg-accent text-phone font-medium text-accent-ink outline-offset-2', blocked && 'cursor-default opacity-40']}
				>{submitLabel ?? m.phone_form_save()}</button
			>
		</div>
	</form>
</BottomSheet>
