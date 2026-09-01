<!-- One space: what it is called, what it protects, and what it serves. -->
<script lang="ts">
  import { SettingRow } from "$lib/ui";
  import { resourceId, type Space } from "$lib/api";
  import ProtectionBadge from "./ProtectionBadge.svelte";

  interface Props {
    /** The space to show. */
    space: Space;
  }

  let { space }: Props = $props();
</script>

<SettingRow
  label={space.displayName}
  description="{resourceId(space.name)}{space.projects.length > 0
    ? ` · serves ${space.projects.length} project${space.projects.length === 1 ? '' : 's'}`
    : ''}"
  tag={space.deleted ? "deleted" : undefined}
>
  {#snippet control()}
    <ProtectionBadge protection={space.protection} locked={space.locked} />
  {/snippet}
</SettingRow>
