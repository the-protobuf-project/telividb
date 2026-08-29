<!--
  The panel switcher.

  Panels with no service behind them are shown and disabled rather than hidden.
  A reader who can see that Graph exists and is not ready knows more than one
  who sees nothing, and the tooltip names what it is waiting on.
-->
<script lang="ts">
  import { Button } from "$lib/components/ui/button";

  /** One entry in the switcher. */
  interface Panel {
    /** Stable key, and the value `active` takes. */
    readonly id: string;
    /** What the reader sees. */
    readonly label: string;
    /** A glyph, kept to one character so the column stays even. */
    readonly glyph: string;
    /** Absent when the panel is ready; the reason it is not, otherwise. */
    readonly blocked?: string;
  }

  interface Props {
    /** Which panel is showing. */
    active: string;
    /** The nav itself, for the launch sequence to move. */
    ref?: HTMLElement | null;
  }

  let { active = $bindable("search"), ref = $bindable(null) }: Props = $props();

  const panels: readonly Panel[] = [
    { id: "search", label: "Search", glyph: "⌕" },
    { id: "collections", label: "Collections", glyph: "◳" },
    { id: "points", label: "Points", glyph: "▤" },
    {
      id: "graph",
      label: "Graph",
      glyph: "⬡",
      blocked: "The Graph service is not served yet.",
    },
    {
      id: "system",
      label: "System",
      glyph: "◈",
      blocked: "The SystemInfo service is not served yet.",
    },
  ];
</script>

<nav
  bind:this={ref}
  class="bg-sidebar flex w-44 shrink-0 flex-col gap-1 border-r p-2"
>
  {#each panels as panel (panel.id)}
    <Button
      variant={active === panel.id ? "secondary" : "ghost"}
      size="sm"
      disabled={!!panel.blocked}
      title={panel.blocked}
      onclick={() => (active = panel.id)}
      class="justify-start gap-2.5"
    >
      <span class="text-muted-foreground w-3 text-center">{panel.glyph}</span>
      {panel.label}
    </Button>
  {/each}
</nav>
