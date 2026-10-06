import { describe, expect, it } from "vitest";
import { groupReasoning, reasoningText } from "./reasoning";
import type { RenderedMessage } from "./types";

let clock = 0;
function msg(partial: Partial<RenderedMessage>): RenderedMessage {
  clock += 1;
  return {
    id: `m${clock}`,
    sender: "assistant",
    content: "",
    content_blocks: null,
    created_at: new Date(Date.UTC(2026, 9, 5, 10, 0, clock)).toISOString(),
    my_feedback: null,
    agent_display_name: null,
    memory_count: 0,
    content_html: "",
    ...partial,
  };
}

describe("reasoningText", () => {
  it("reads the reasoning block and the older 🧠 form", () => {
    expect(
      reasoningText(
        msg({
          content: "🧠 x",
          content_blocks: [{ kind: "reasoning", text: "from block" }],
        }),
      ),
    ).toBe("from block");
    expect(reasoningText(msg({ content: "🧠 legacy step" }))).toBe(
      "legacy step",
    );
    expect(reasoningText(msg({ content: "a normal reply" }))).toBeNull();
    expect(
      reasoningText(msg({ sender: "user", content: "🧠 user typed this" })),
    ).toBeNull();
  });
});

describe("groupReasoning", () => {
  it("folds thinking into the same agent’s next reply", () => {
    const user = msg({ sender: "user", content: "plan my trip" });
    const t1 = msg({ content: "🧠 first", agent_display_name: "Planning" });
    const t2 = msg({ content: "🧠 second", agent_display_name: "Planning" });
    const reply = msg({
      content: "Here is the plan",
      agent_display_name: "Planning",
    });
    const items = groupReasoning([user, t1, t2, reply]);
    expect(items.map((i) => i.message.id)).toEqual([user.id, reply.id]);
    expect(items[1].reasoning).toEqual(["first", "second"]);
  });

  it("keeps one agent’s thinking off another agent’s reply", () => {
    const thinking = msg({
      content: "🧠 money thoughts",
      agent_display_name: "Money",
    });
    const nomi = msg({ content: "Nomi here" });
    const items = groupReasoning([thinking, nomi]);
    expect(items[0]).toMatchObject({
      message: thinking,
      reasoning: ["money thoughts"],
      thinkingOnly: true,
    });
    expect(items[1]).toMatchObject({ message: nomi, reasoning: [] });
  });

  it("shows thinking that has no reply yet where it started", () => {
    const user = msg({ sender: "user", content: "hi" });
    const thinking = msg({ content: "🧠 hmm" });
    const items = groupReasoning([user, thinking]);
    expect(items.at(-1)).toMatchObject({
      thinkingOnly: true,
      reasoning: ["hmm"],
    });
  });
});
