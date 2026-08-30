<!--
  What a space's protection actually promises.

  The wording is deliberately unequal, because the guarantees are. "Private" is
  an owner predicate the server enforces and anyone who compromises the server
  defeats; "vault" and "sealed" are cryptographic. Rule 25 exists because it is
  tempting to call all three the same reassuring word.
-->
<script lang="ts">
  import type { Protection } from "@telividb/answer";

  interface Props {
    /** The protection to describe. */
    protection: Protection;
    /** Whether the space's key is currently unavailable. */
    locked?: boolean;
  }

  let { protection, locked = false }: Props = $props();

  const described: Record<Protection, { label: string; title: string }> = {
    none: {
      label: "open",
      title: "Visible according to ordinary role grants on its projects.",
    },
    private: {
      label: "private",
      title:
        "Readable only by its owner, enforced by a visibility predicate. Access control, not cryptography.",
    },
    vault: {
      label: "vault",
      title: "Encrypted with a key the server wraps and holds.",
    },
    sealed: {
      label: "sealed",
      title:
        "Encrypted with a key only the client holds. The server cannot read it even when compromised.",
    },
  };

  let it = $derived(described[protection]);
</script>

<span class="flex items-center gap-1.5">
  <span
    class="border-border text-muted-foreground border px-1.5 py-0.5 text-[0.6875rem]"
    title={it.title}
  >
    {it.label}
  </span>
  {#if locked}
    <span
      class="border-[var(--color-amber)]/40 text-[var(--color-amber)] border px-1.5 py-0.5 text-[0.6875rem]"
      title="Its key is unavailable, so nothing in it is searchable. A search elsewhere reports that results may be incomplete rather than quietly leaving it out."
    >
      locked
    </span>
  {/if}
</span>
