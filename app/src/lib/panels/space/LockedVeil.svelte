<!--
  A space whose key is not available this session.

  Covers the thread rather than emptying it, because "nothing here" and "nothing
  you can currently see" are different facts and only one of them is the
  reader's to act on (rule 27). The unlock controls are present and disabled:
  recognition would release a key already on this machine, and neither the
  policy engine nor the vault it would open exists yet.
-->
<script lang="ts">
  import { Button } from "$lib/ui";

  interface Props {
    /** The space's display name. */
    name: string;
    /** Whether it is sealed rather than server-wrapped. */
    sealed?: boolean;
  }

  let { name, sealed = false }: Props = $props();
</script>

<div class="locked-veil">
  <div class="locked-inner">
    <svg width="26" height="26" viewBox="0 0 12 12" fill="none" stroke="var(--amber)" stroke-width="1">
      <rect x="2.5" y="5.5" width="7" height="5" />
      <path d="M4 5.5V4a2 2 0 0 1 4 0v1.5" />
    </svg>
    <div>
      <h2>{name} is locked</h2>
      <p class="lede">
        Its contents are {sealed ? "encrypted with a key only you hold" : "key-wrapped"}.
        Nothing in it is searchable — and a search elsewhere reports that results
        may be incomplete rather than quietly leaving it out.
      </p>
    </div>
    <div style="display: flex; gap: calc(var(--u) * 2)">
      <Button disabled title="Needs the policy engine">Unlock with face</Button>
      <Button variant="ghost" disabled title="Needs the policy engine">Use voice</Button>
    </div>
  </div>
</div>
