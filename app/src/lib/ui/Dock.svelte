<!--
  The panel switcher: a 52px icon rail, exactly as the design has it.

  Icons are inline SVG carried over from the mock rather than an icon font. They
  are drawn at 17px on a 18px grid with a 1.3 stroke, and an icon set swapped in
  for them would change every weight and corner on this rail — which is most of
  what the dock looks like.

  A blocked panel is shown and disabled rather than hidden: a reader who can see
  that People exists and is not ready knows more than one who sees nothing.
-->
<script lang="ts">
  interface Entry {
    /** Stable key, and the value `active` takes. */
    readonly id: string;
    /** Shown in the hover tip and to a screen reader. */
    readonly label: string;
    /** The icon's SVG body, on an 18x18 grid. */
    readonly path: string;
    /** Absent when the panel is ready; the reason it is not, otherwise. */
    readonly blocked?: string;
  }

  interface Props {
    /** Which panel is showing. */
    active: string;
    /** The dock itself, for the launch sequence to move. */
    ref?: HTMLElement | null;
  }

  let { active = $bindable("workspace"), ref = $bindable(null) }: Props = $props();

  const entries: readonly Entry[] = [
    { id: "workspace", label: "Workspace", path: `<path d="M2.5 3.5h13v9h-7l-3.5 3v-3h-2.5z"/>` },
    { id: "data", label: "Data", path: `<ellipse cx="9" cy="4.5" rx="6" ry="2.2"/><path d="M3 4.5v9c0 1.2 2.7 2.2 6 2.2s6-1 6-2.2v-9"/><path d="M3 9c0 1.2 2.7 2.2 6 2.2s6-1 6-2.2"/>` },
    { id: "models", label: "Models", path: `<path d="M9 2.5v9M5.5 8L9 11.5 12.5 8"/><path d="M3 12.5v2h12v-2"/>` },
    { id: "people", label: "People", path: `<circle cx="6.8" cy="6" r="2.6"/><path d="M2 15c0-2.7 2.2-4.4 4.8-4.4S11.6 12.3 11.6 15"/><path d="M12 4.2a2.6 2.6 0 0 1 0 5"/><path d="M13 10.8c2.1.3 3.6 1.9 3.6 4.2"/>` },
    { id: "metrics", label: "Metrics", path: `<path d="M2.5 14.5v-4M6.8 14.5v-8M11.2 14.5v-5.5M15.5 14.5v-11"/>` },
    { id: "settings", label: "Settings", path: `<circle cx="9" cy="9" r="2.4"/><path d="M9 1.8v1.9M9 14.3v1.9M16.2 9h-1.9M3.7 9H1.8M14.1 3.9l-1.3 1.3M5.2 12.8l-1.3 1.3M14.1 14.1l-1.3-1.3M5.2 5.2L3.9 3.9"/>` }
  ];
</script>

<nav class="dock" bind:this={ref}>
  {#each entries as entry (entry.id)}
    <button
      class="dock-btn"
      type="button"
      aria-current={active === entry.id}
      aria-label={entry.label}
      disabled={!!entry.blocked}
      title={entry.blocked}
      onclick={() => (active = entry.id)}
    >
      <svg
        width="17"
        height="17"
        viewBox="0 0 18 18"
        fill="none"
        stroke="currentColor"
        stroke-width="1.3"
      >
        {@html entry.path}
      </svg>
      <span class="tip">{entry.blocked ?? entry.label}</span>
    </button>
  {/each}
</nav>
