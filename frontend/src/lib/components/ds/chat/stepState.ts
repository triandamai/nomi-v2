import { Check, CircleAlert, CircleDashed, LoaderCircle } from '@lucide/svelte';

/** Where a step of the crew's work is, as shown in plans and step lists. */
export type StepState = 'done' | 'running' | 'waiting' | 'failed';

export const STEP_ICON = { done: Check, running: LoaderCircle, waiting: CircleDashed, failed: CircleAlert } as const;

/** Two-digit step numbers, in mono: 01, 02, … */
export const stepNumber = (index: number) => String(index + 1).padStart(2, '0');
