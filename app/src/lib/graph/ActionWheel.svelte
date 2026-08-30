<!--
  Quick actions, arranged around the node they act on.

  A wheel rather than a menu because the cursor is already on the node: every
  action is the same short distance away in a different direction, and the
  direction is learnable in a way a list position is not.

  Words, not glyphs. Six icons around a node is six things to learn before the
  menu beats a list, and half of these — pin, expand, open — have no icon two
  people would read the same way. The ring is sized so nothing touches the node
  it acts on: the point of the wheel is to reach an action without losing sight
  of what it applies to.
-->
<script lang="ts">
  import { NODE_ACTIONS, seat, type NodeAction } from "./actions";

  interface Props {
    /** Where the node is, in the viewport. */
    x: number;
    /** Where the node is, in the viewport. */
    y: number;
    /** What the wheel acts on, named in the centre. */
    label: string;
    /** The actions. Defaults to the six for a point. */
    actions?: readonly NodeAction[];
    /** Called with the chosen action's id. */
    onpick: (id: string) => void;
    /** Called when the wheel is dismissed without a choice. */
    onclose: () => void;
  }

  let { x, y, label, actions = NODE_ACTIONS, onpick, onclose }: Props = $props();

  /**
   * The ring, shaped like the thing it surrounds and clear of it.
   *
   * A node is 176×32, so its half-width is 88. At 60° the seat is at
   * `0.866 × RX`, and a text button is about 68 wide — so `0.866 × 168 − 34 =
   * 111`, leaving 23px of air beside the node. `RY` is set the same way against
   * the 16px half-height. An ellipse rather than a circle because a circle wide
   * enough to clear the sides is far too tall to read as belonging to the node.
   */
  const RX = 168;
  const RY = 74;

  let hovered = $state<string | null>(null);

  /** The action under the pointer, explained — or the node's own name. */
  let caption = $derived(actions.find((a) => a.id === hovered)?.detail ?? label);
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape") onclose();
  }}
/>

<!-- A backdrop that only catches clicks: dismissing by clicking away is what
     makes the wheel safe to open, and without it every stray click would land
     on the canvas underneath. -->
<div
  class="wheel-veil"
  role="presentation"
  onclick={onclose}
  oncontextmenu={(e) => {
    e.preventDefault();
    onclose();
  }}
></div>

<div class="wheel" style="left: {x}px; top: {y}px" role="menu" aria-label="Actions for {label}">
  <span class="wheel-caption">{caption}</span>

  {#each actions as action, i (action.id)}
    {@const at = seat(i, actions.length, RX, RY)}
    <button
      class="wheel-btn"
      type="button"
      role="menuitem"
      style="transform: translate({at.x}px, {at.y}px)"
      disabled={action.enabled === false}
      title={action.detail}
      onmouseenter={() => (hovered = action.id)}
      onmouseleave={() => (hovered = null)}
      onfocus={() => (hovered = action.id)}
      onblur={() => (hovered = null)}
      onclick={() => onpick(action.id)}
    >
      {action.label}
    </button>
  {/each}
</div>
