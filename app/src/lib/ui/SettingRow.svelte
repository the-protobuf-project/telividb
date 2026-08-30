<!--
  One setting: what it is, what it means, and the control that changes it.

  The description is not optional decoration. Most of these settings are choices
  about where data goes, and a label alone ("Redact before embedding") tells a
  reader what the switch is called rather than what happens if they touch it.
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    /** What the setting is called. */
    label: string;
    /** What it means, or what breaks without it. */
    description?: string;
    /** Shown beside the label — "local", "not wired", and so on. */
    tag?: string;
    /** The control itself. */
    control: Snippet;
  }

  let { label, description, tag, control }: Props = $props();
</script>

<div class="border-border flex items-center justify-between gap-6 border-b px-4 py-3 last:border-b-0">
  <div class="flex min-w-0 flex-col gap-0.5">
    <span class="text-foreground flex items-center gap-2 text-sm font-medium">
      {label}
      {#if tag}
        <span class="border-border text-muted-foreground border px-1.5 py-0.5 text-[0.6875rem]">
          {tag}
        </span>
      {/if}
    </span>
    {#if description}
      <span class="text-muted-foreground text-xs">{description}</span>
    {/if}
  </div>
  <div class="flex shrink-0 items-center gap-2">{@render control()}</div>
</div>
