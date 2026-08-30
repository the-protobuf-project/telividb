<!-- One space: what it is called, what it protects, and what it serves. -->
<script lang="ts">
  import { resourceId, type Space } from "$lib/api";
  import ProtectionBadge from "./ProtectionBadge.svelte";

  interface Props {
    /** The space to show. */
    space: Space;
  }

  let { space }: Props = $props();
</script>

<div
  class="border-border flex items-center justify-between gap-4 border-b px-3 py-2 last:border-b-0 {space.deleted
    ? 'opacity-50'
    : ''}"
>
  <div class="flex min-w-0 flex-col gap-0.5">
    <span class="text-foreground flex items-center gap-2 text-sm">
      {space.displayName}
      {#if space.deleted}
        <span class="text-muted-foreground text-xs">deleted</span>
      {/if}
    </span>
    <span class="text-muted-foreground font-mono text-xs">
      {resourceId(space.name)}
      {#if space.projects.length > 0}
        · serves {space.projects.length} project{space.projects.length === 1 ? "" : "s"}
      {/if}
    </span>
  </div>
  <ProtectionBadge protection={space.protection} locked={space.locked} />
</div>
