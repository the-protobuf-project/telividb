<!--
  The one fact no orchestrator can see from outside this process.

  A build that quietly fell back to the CPU passes every correctness test while
  delivering none of the speed, so the selected backend is stated rather than
  inferred. Until the SystemInfo service is served, this says the backend is not
  readable — which is true — instead of guessing at one, because a wrong badge
  here would be worse than an absent one.
-->
<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import { Separator } from "$lib/components/ui/separator";

  interface Props {
    /** Where the engine is listening, once the window has asked. */
    address: string | null;
    /** The bar itself, for the launch sequence to move. */
    ref?: HTMLElement | null;
    /** The wordmark, which the sequence brings in first. */
    markRef?: HTMLElement | null;
  }

  let {
    address = null,
    ref = $bindable(null),
    markRef = $bindable(null),
  }: Props = $props();
</script>

<header
  bind:this={ref}
  class="flex shrink-0 items-center gap-3 border-b px-4 py-2"
>
  <span bind:this={markRef} class="text-sm font-medium tracking-tight">
    telividb
  </span>

  <div class="ml-auto flex items-center gap-3">
    {#if address}
      <span class="text-muted-foreground tnum selectable text-xs">
        {address}
      </span>
      <Separator orientation="vertical" class="h-4" />
    {/if}
    <Badge
      variant="outline"
      class="text-muted-foreground gap-1.5 font-normal"
      title="Reported by the SystemInfo service, which is not served yet."
    >
      <span class="bg-muted-foreground/60 size-1.5 rounded-full"></span>
      backend unknown
    </Badge>
  </div>
</header>
