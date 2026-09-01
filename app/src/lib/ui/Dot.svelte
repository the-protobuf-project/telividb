<!--
  A status dot.

  Four states rather than a boolean, because "not configured" and "failed" are
  different facts and a single grey/green pair cannot tell them apart.

  # It says its state in words, not only in colour

  The dot is an empty box whose entire meaning was its fill, with a `title` as
  the only alternative — and a `title` on a span with no role is not reliably
  announced. It now carries `role="img"` and a name, so the state survives for
  a reader who cannot see the colour, is not using a pointer, or has a red
  deficiency.

  Beside its own label — inside a `Chip`, or a row that already says "installed"
  — pass `decorative` instead: naming it twice is noise, and the icon guidance
  is explicit that something decorative beside visible text should be hidden.
-->
<script lang="ts">
  /** What the dot reports. */
  export type DotState = "idle" | "live" | "warn" | "off";

  interface Props {
    /** What the dot reports. Idle is the unconfigured default. */
    state?: DotState;
    /**
     * What the state means here, in words.
     *
     * Worth writing per use rather than defaulting: "live" means "on disk" on a
     * model row and "runs on this machine" on a provider, and a generic name
     * would be worse than none.
     */
    title?: string;
    /** Beside text that already says this. Hides it from assistive tech. */
    decorative?: boolean;
  }

  let { state = "idle", title, decorative = false }: Props = $props();

  /** A fallback name, so the state is never announced as nothing at all. */
  const named: Record<DotState, string> = {
    idle: "not configured",
    live: "ready",
    warn: "attention",
    off: "failed",
  };
</script>

<span
  class="dot {state === 'idle' ? '' : state}"
  role={decorative ? undefined : "img"}
  aria-hidden={decorative ? "true" : undefined}
  aria-label={decorative ? undefined : (title ?? named[state])}
  {title}
></span>
