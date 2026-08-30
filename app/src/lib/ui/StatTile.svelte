<!--
  One measured number, with its unit and what it was measured over.

  A number on its own is not a fact — 0.42 could be milliseconds or a score, and
  "over 12 points" is what makes it mean something. So the note is a first-class
  slot rather than an afterthought, and the value is tabular so a column of
  these lines up.
-->
<script lang="ts">
  interface Props {
    /** What is being measured. */
    label: string;
    /** The measurement. `—` when nothing has been recorded yet. */
    value: string | number;
    /** Its unit, set smaller beside the number. */
    unit?: string;
    /** What it was measured over — the context that makes the number readable. */
    note?: string;
  }

  let { label, value, unit, note }: Props = $props();
</script>

<div class="bg-card flex flex-col gap-1 px-3.5 py-3">
  <span class="text-muted-foreground text-[0.6875rem] tracking-wide uppercase">{label}</span>
  <span class="tnum text-[1.375rem] leading-none tracking-tight">
    {value}{#if unit}<small class="text-muted-foreground ml-0.5 text-[0.8125rem]">{unit}</small>{/if}
  </span>
  {#if note}
    <span class="text-muted-foreground text-[0.6875rem]">{note}</span>
  {/if}
</div>
