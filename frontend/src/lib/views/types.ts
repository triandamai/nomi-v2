// Views: what the crew shows in a chat bubble when plain text isn't the best fit. Mirrors the
// backend's catalog (backend/crates/nomi-agent-core/src/views.rs); examples.json holds one of
// each, parsed by both sides' tests. To add a view, see the steps at the top of that file.

export type Format = 'text' | 'number' | 'money' | 'percent' | 'date' | 'date_time';
export type Scalar = string | number | boolean;
export type Tone = 'neutral' | 'positive' | 'negative' | 'warning';

export interface Field {
	label: string;
	value: Scalar;
	format?: Format;
}

export interface ListView {
	view: 'list';
	title?: string;
	style?: 'bullet' | 'numbered' | 'check';
	items: { title: string; detail?: string; meta?: Scalar; format?: Format; done?: boolean; url?: string }[];
}

export interface TableView {
	view: 'table';
	title?: string;
	columns: { key: string; label: string; format?: Format }[];
	rows: Record<string, unknown>[];
}

export interface ComparisonView {
	view: 'comparison';
	title?: string;
	options: string[];
	rows: { label: string; values: Scalar[] }[];
	recommended?: number;
	verdict?: string;
}

export type StepStatus = 'done' | 'current' | 'upcoming' | 'failed' | 'skipped';
export interface StepperView {
	view: 'stepper';
	title?: string;
	steps: { title: string; detail?: string; status?: StepStatus }[];
}

export interface TimelineView {
	view: 'timeline';
	title?: string;
	events: { at: string; title: string; detail?: string }[];
}

export interface CardView {
	view: 'card';
	eyebrow?: string;
	title: string;
	subtitle?: string;
	highlight?: Field;
	fields?: Field[];
	body?: string;
	link?: { label: string; url: string };
	actions?: { label: string; message: string }[];
	tone?: Tone;
}

export interface ChartView {
	view: 'chart';
	title?: string;
	chart?: 'bar' | 'line' | 'donut';
	labels: string[];
	series: { name: string; values: number[] }[];
	format?: Format;
}

export interface AnswerView {
	view: 'answer';
	label?: string;
	value: Scalar;
	format?: Format;
	detail?: string;
	tone?: Tone;
}

export interface SuggestionsView {
	view: 'suggestions';
	title?: string;
	items: string[];
}

export interface DecisionView {
	view: 'decision';
	question: string;
	options: { label: string; detail?: string; pros?: string[]; cons?: string[]; recommended?: boolean }[];
}

export type FormFieldType = 'text' | 'textarea' | 'number' | 'date' | 'time' | 'select' | 'multiselect' | 'checkbox';
export interface FormField {
	key: string;
	label: string;
	type?: FormFieldType;
	required?: boolean;
	options?: string[];
	placeholder?: string;
	default?: Scalar;
}
export interface FormView {
	view: 'form';
	title?: string;
	intro?: string;
	fields: FormField[];
	submit_label?: string;
}

export interface LinkView {
	view: 'link';
	url: string;
	title?: string;
	description?: string;
	image?: string;
	site?: string;
}

export interface TaskView {
	view: 'task';
	title: string;
	status?: 'queued' | 'running' | 'done' | 'failed';
	detail?: string;
	progress?: number;
	agent?: string;
}

export interface ShellView {
	view: 'shell';
	command: string;
	output?: string;
	exit_code?: number;
	cwd?: string;
}

export type View =
	| ListView
	| TableView
	| ComparisonView
	| StepperView
	| TimelineView
	| CardView
	| ChartView
	| AnswerView
	| SuggestionsView
	| DecisionView
	| FormView
	| LinkView
	| TaskView
	| ShellView;
export type ViewName = View['view'];

/** What every view component gets: its data, and the chat it sits in. */
export interface ViewProps<V extends View = View> {
	view: V;
	/** The crew member who showed it. */
	agent?: string | null;
	/** The person has replied since (a decision or form shows as answered). */
	answered?: boolean;
}
