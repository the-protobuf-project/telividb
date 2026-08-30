<!--
  A small status readout: a dot and a word.

  The dot is the whole point. A chip that only carried text would need reading;
  a coloured dot is scannable from across the desk, which is what a status
  readout is for. Colour never carries the meaning alone — the label is always
  there beside it.
-->
<script lang="ts">
  /** What a chip is reporting. */
  export type ChipState = "live" | "warn" | "off" | "idle";

  interface Props {
    /** The word shown. Kept short — this is a readout, not a sentence. */
    label: string;
    /** What the dot reports. `idle` is the unlit default. */
    state?: ChipState;
    /** Hover text, for the fact the label had no room to say. */
    title?: string;
  }

  let { label, state = "idle", title }: Props = $props();

  const dot: Record<ChipState, string> = {
    live: "bg-[--color-green]",
    warn: "bg-[--color-amber]",
    off: "bg-[--color-red]",
    idle: "bg-[--color-ink-faint]",
  };
</script>

<span
  class="border-border bg-background text-muted-foreground inline-flex h-7 items-center gap-1.5 border px-2 text-xs whitespace-nowrap"
  {title}
>
  <span class="size-1.5 shrink-0 {dot[state]}"></span>
  {label}
</span>
