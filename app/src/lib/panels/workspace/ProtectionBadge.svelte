<!--
  What a space's protection promises, and whether its key is available.

  Two separate facts: `Lock` says what kind of protection this is, the `locked`
  tag says whether it can be read right now. Collapsing them would lose the
  difference between "encrypted" and "encrypted and currently unopenable".
-->
<script lang="ts">
  import { Lock, Tag, type Protection } from "$lib/ui";

  interface Props {
    /** The protection to describe. */
    protection: Protection;
    /** Whether the space's key is currently unavailable. */
    locked?: boolean;
  }

  let { protection, locked = false }: Props = $props();
</script>

<span style="display: flex; align-items: center; gap: calc(var(--u) * 1.5)">
  <Lock {protection} decorative />
  <span class="faint" style="font-size: 0.6875rem">{protection}</span>
  {#if locked}
    <Tag
      tone="amber"
      title="Its key is unavailable, so nothing in it is searchable. A search elsewhere reports that results may be incomplete rather than quietly leaving it out."
    >
      locked
    </Tag>
  {/if}
</span>
