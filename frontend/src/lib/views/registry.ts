import type { Component } from 'svelte';
import AnswerView from '$lib/components/views/AnswerView.svelte';
import CardView from '$lib/components/views/CardView.svelte';
import ChartView from '$lib/components/views/ChartView.svelte';
import ComparisonView from '$lib/components/views/ComparisonView.svelte';
import DecisionView from '$lib/components/views/DecisionView.svelte';
import FormView from '$lib/components/views/FormView.svelte';
import LinkView from '$lib/components/views/LinkView.svelte';
import ListView from '$lib/components/views/ListView.svelte';
import ShellView from '$lib/components/views/ShellView.svelte';
import StepperView from '$lib/components/views/StepperView.svelte';
import SuggestionsView from '$lib/components/views/SuggestionsView.svelte';
import TableView from '$lib/components/views/TableView.svelte';
import TaskView from '$lib/components/views/TaskView.svelte';
import TimelineView from '$lib/components/views/TimelineView.svelte';
import type { View, ViewName, ViewProps } from './types';

// Which component draws each view. Typed so that a view in `View` without a component here
// doesn't compile (names.ts lists the same views for the tests, which can't load components).
export const VIEW_COMPONENTS: { [K in ViewName]: Component<ViewProps<Extract<View, { view: K }>>> } = {
	list: ListView,
	table: TableView,
	comparison: ComparisonView,
	stepper: StepperView,
	timeline: TimelineView,
	card: CardView,
	chart: ChartView,
	answer: AnswerView,
	suggestions: SuggestionsView,
	decision: DecisionView,
	form: FormView,
	link: LinkView,
	task: TaskView,
	shell: ShellView,
};

/** The component for a view, or none for a view this build doesn't know (from a newer backend). */
export function viewComponent(view: { view: string }): Component<ViewProps> | null {
	return (VIEW_COMPONENTS as Record<string, Component<ViewProps>>)[view.view] ?? null;
}
