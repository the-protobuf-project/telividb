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

<div class="set-row" style={space.deleted ? "opacity:.5" : ""}>
  <div class="txt">
    <b>{space.displayName}</b>
    <span>
      {resourceId(space.name)}
      {#if space.projects.length > 0}
        · serves {space.projects.length} project{space.projects.length === 1 ? "" : "s"}
      {/if}
    </span>
  </div>
  <div class="ctl">
    <ProtectionBadge protection={space.protection} locked={space.locked} />
  </div>
</div>
