import type { Component } from 'svelte';
import type { LucideProps } from '@lucide/svelte';

// The app's toasts: show one from anywhere, ToastRegion lays them out. They go away on their own
// after a while (`duration`, e.g. 10 s for an Undo), except errors, which stay until dismissed.

export type ToastInput = {
	kind: 'success' | 'error' | 'info';
	title: string;
	detail?: string;
	icon?: Component<LucideProps>;
	action?: { label: string; onclick: () => void };
	/** Milliseconds before it goes. Default 6 s; errors stay unless this is given. */
	duration?: number;
};

const DEFAULT_DURATION = 6000;

class Toasts {
	items = $state<(ToastInput & { id: number })[]>([]);
	#next = 1;
	#timers = new Map<number, ReturnType<typeof setTimeout>>();

	show(toast: ToastInput): number {
		const id = this.#next++;
		this.items.push({ ...toast, id });
		const duration = toast.duration ?? (toast.kind === 'error' ? undefined : DEFAULT_DURATION);
		if (duration) this.#timers.set(id, setTimeout(() => this.dismiss(id), duration));
		return id;
	}

	dismiss(id: number) {
		clearTimeout(this.#timers.get(id));
		this.#timers.delete(id);
		this.items = this.items.filter((toast) => toast.id !== id);
	}

	clear() {
		for (const timer of this.#timers.values()) clearTimeout(timer);
		this.#timers.clear();
		this.items = [];
	}
}

export const toasts = new Toasts();
