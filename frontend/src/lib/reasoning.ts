// Folds an agent's reasoning messages into the reply they led to, so the thread shows one
// "Thought process" disclosure on that reply instead of a separate message per thinking step.

import type { RenderedMessage } from "./types";

const LEGACY_PREFIX = "🧠 ";

/** The thinking text if `message` is a reasoning step, else null. Older steps were plain text
 * starting with 🧠 and carry no block. */
export function reasoningText(message: RenderedMessage): string | null {
  if (message.sender !== "assistant") return null;
  const block = message.content_blocks?.[0];
  if (block?.kind === "reasoning") return block.text;
  if (
    !message.content_blocks?.length &&
    message.content.startsWith(LEGACY_PREFIX)
  ) {
    return message.content.slice(LEGACY_PREFIX.length);
  }
  return null;
}

export interface ThreadItem {
  message: RenderedMessage;
  /** Thinking steps that led to this message, oldest first. */
  reasoning: string[];
  /** True when the item is only thinking so far: no reply from that agent has landed yet. */
  thinkingOnly: boolean;
}

export function groupReasoning(messages: RenderedMessage[]): ThreadItem[] {
  const items: ThreadItem[] = [];
  // Steps waiting for their reply, per agent (display name; null = Nomi).
  const pending = new Map<
    string | null,
    { steps: string[]; first: RenderedMessage }
  >();

  for (const message of messages) {
    const text = reasoningText(message);
    const agent = message.agent_display_name ?? null;
    if (text !== null) {
      const waiting = pending.get(agent);
      if (waiting) waiting.steps.push(text);
      else pending.set(agent, { steps: [text], first: message });
      continue;
    }
    const waiting =
      message.sender === "assistant" ? pending.get(agent) : undefined;
    if (waiting) pending.delete(agent);
    items.push({
      message,
      reasoning: waiting?.steps ?? [],
      thinkingOnly: false,
    });
  }

  // Thinking with no reply yet (still working, or stopped) stays visible, collapsed, where it began.
  for (const { steps, first } of pending.values()) {
    const at = items.findIndex(
      (item) => item.message.created_at > first.created_at,
    );
    const item = { message: first, reasoning: steps, thinkingOnly: true };
    if (at === -1) items.push(item);
    else items.splice(at, 0, item);
  }
  return items;
}
