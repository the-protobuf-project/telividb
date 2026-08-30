<!-- One tab of the right-hand detail panel. -->
<script lang="ts">
  interface Props {
    /** The tab's name. */
    label: string;
    /** Whether its panel is showing. */
    current?: boolean;
    /** Clicking shows it. */
    onclick?: () => void;
  }

  let { label, current = false, onclick }: Props = $props();
</script>

<button class="side-tab" type="button" aria-current={current} {onclick}>
  {label}
</button>
