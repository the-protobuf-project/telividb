<!--
  The padlock beside a space, and what it actually promises.

  # It names the protection, not only draws it

  A glyph with a `title` was the whole signal for a space's protection in the
  rail — and the difference between `vault` and `sealed` is the difference
  between "the server holds the key" and "it cannot read this at all". That is
  not something to leave to a shape a reader may not be able to see.

  Beside its own label, in the rail's legend, pass `decorative`.

  Three glyphs for four states, and the wording is deliberately unequal because
  the guarantees are: `private` is a predicate the server enforces and anyone who
  compromises the server defeats, while `vault` and `sealed` are cryptographic.
  Rule 25 exists because it is tempting to draw all three the same.
-->
<script lang="ts">
  /** How a space is protected, in the wire's own vocabulary. */
  export type Protection = "none" | "private" | "vault" | "sealed";

  interface Props {
    /** The protection to draw. */
    protection: Protection;
    /** Beside text that already names it. Hides it from assistive tech. */
    decorative?: boolean;
  }

  let { protection, decorative = false }: Props = $props();

  const drawn: Record<Protection, { glyph: string; cls: string; title: string }> = {
    none: {
      glyph: "○",
      cls: "lock-open",
      title: "Visible according to ordinary role grants on its projects.",
    },
    private: {
      glyph: "○",
      cls: "lock-open",
      title:
        "Readable only by its owner, enforced by a visibility predicate. Access control, not cryptography.",
    },
    vault: {
      glyph: "◐",
      cls: "lock-shut",
      title: "Encrypted with a key the server wraps and holds.",
    },
    sealed: {
      glyph: "●",
      cls: "lock-sealed",
      title:
        "Encrypted with a key only the client holds. The server cannot read it even when compromised.",
    },
  };

  let it = $derived(drawn[protection]);
</script>

<span
  class="lock {it.cls}"
  role={decorative ? undefined : "img"}
  aria-hidden={decorative ? "true" : undefined}
  aria-label={decorative ? undefined : `${protection} — ${it.title}`}
  title={it.title}
>{it.glyph}</span>
