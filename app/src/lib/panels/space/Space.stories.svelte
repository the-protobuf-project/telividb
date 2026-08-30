<!--
  The working surface: a space, its conversation, and the box you add to it.

  Three states, because the third is the one a happy-path drawing never
  includes — a vault whose key is not available this session covers the thread
  rather than emptying it, so "nothing here" and "nothing you can currently see"
  stay distinguishable.
-->
<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import Space from "./Space.svelte";
  import type { AskState } from "$lib/panels/ask/state.svelte";

  const { Story } = defineMeta({ title: "Workspace/Space", parameters: { layout: "fullscreen" } });

  /** A conversation, shaped like the real one but needing no engine. */
  function fixture(turns: number): AskState {
    const hits = [
      { id: "p1", score: 0.92, text: "Sealed segments are never written again — mutation is a new segment plus a tombstone." },
      { id: "p2", score: 0.74, text: "Compaction is what physically removes a tombstoned row." },
      { id: "p3", score: 0.41, text: "HNSW degrades on delete, so the graph is rebuilt rather than edited in place." },
    ];
    const history = Array.from({ length: turns }, (_, i) => ({
      question: i === 0 ? "How does a delete actually work?" : "And what does compaction cost?",
      hits,
      passages: hits,
      text:
        i === 0
          ? "A delete writes a tombstone rather than touching the segment [1]. The row stays on disk until compaction rewrites the segment without it [2]."
          : "",
      streaming: i > 0,
      error: null,
      copyable: i === 0,
      stop: () => {},
    }));
    return {
      history,
      draft: "",
      asking: false,
      providers: [],
      provider: null,
      model: "",
      ask: () => {},
      loadProviders: async () => {},
    } as unknown as AskState;
  }
</script>

<!-- A conversation in progress: one answered turn, one still arriving. -->
<Story name="Conversation">
  <div class="view workspace" style="height: 100vh; grid-template-columns: 1fr">
    <Space name="Notes" protection="none" ask={fixture(2)} dimensions={384} />
  </div>
</Story>

<!-- Before anything has been asked, which is every space on its first day. -->
<Story name="Empty">
  <div class="view workspace" style="height: 100vh; grid-template-columns: 1fr">
    <Space name="Journal" protection="private" ask={fixture(0)} dimensions={384} />
  </div>
</Story>

<!-- Locked: the state that must never be mistaken for empty. -->
<Story name="Locked vault">
  <div class="view workspace" style="height: 100vh; grid-template-columns: 1fr">
    <Space name="Board" protection="vault" locked ask={fixture(0)} />
  </div>
</Story>
