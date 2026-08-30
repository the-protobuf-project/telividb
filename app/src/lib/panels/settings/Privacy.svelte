<!--
  Privacy, and the controls that do not exist yet.

  Every row is disabled and says what it is waiting on. Three switches that
  looked settable and did nothing would be worse than none: a person who thinks
  they turned redaction on has been told something false about where their data
  goes, and this is the screen where that matters most.
-->
<script lang="ts">
  /** A setting that is designed but has nothing behind it yet. */
  interface Pending {
    /** What it will be called. */
    readonly label: string;
    /** What it will do. */
    readonly description: string;
    /** Which piece of the system has to exist first. */
    readonly waiting: string;
  }

  const pending: readonly Pending[] = [
    {
      label: "Send telemetry",
      // Scoped to telemetry on purpose. The blanket version of this sentence —
      // "nothing leaves this machine" — was written before answering moved into
      // the window, and stopped being true the moment a remote provider could be
      // configured. A privacy screen that overstates is worse than one that says
      // less.
      description:
        "Off. No traces or metrics leave this machine unless you point them at a collector.",
      waiting: "telemetry.toml decides this, not the window.",
    },
    {
      label: "Unlock with face or voice",
      description:
        "Recognition would release a key already on this machine. It never authorizes on its own.",
      waiting: "Needs the policy engine and a vault to unlock.",
    },
    {
      label: "Redact before embedding",
      description:
        "Sensitive spans removed before a model sees them — a vector made from a secret leaks it.",
      waiting: "Needs the redaction pass in the ingest path.",
    },
  ];
</script>

<div>
  <!-- Stated before the switches, because it is the one thing on this screen
       that is true right now: a configured remote provider receives the question
       and the passages retrieved for it. Retrieval itself never leaves. -->
  <p class="hint" style="margin-bottom: 0.625rem">
    Retrieval always happens on this machine. Answering does not: a remote
    provider configured under Answering is sent the question and the passages
    found for it, and is shown on every turn before you send.
  </p>

  <div class="set-group" style="opacity: 0.6">
    {#each pending as row (row.label)}
      <div class="set-row">
        <div class="txt">
          <b>{row.label} <span class="tag">not built</span></b>
          <span>{row.description}</span>
        </div>
        <div class="ctl faint" style="font-size: 0.75rem; max-width: 14rem; text-align: right">
          {row.waiting}
        </div>
      </div>
    {/each}
  </div>
</div>
