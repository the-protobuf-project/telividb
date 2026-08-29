<!--
  One icon.

  Lineicons ships a webfont, so an icon is a class on an empty element rather
  than an inline SVG. Wrapping that in a component means the markup says what
  the icon *is* — `<Icon name="database" />` — instead of repeating a font
  class, and a change of icon set later is one file rather than every call site.
-->
<script lang="ts">
  interface Props {
    /** The Lineicons name, without the `lni-` prefix. */
    name: string;
    /** Extra classes, for size or colour. */
    class?: string;
    /**
     * What a screen reader should say.
     *
     * Absent by default, which marks the icon decorative — correct when it sits
     * beside a label, and wrong when it stands alone.
     */
    label?: string;
  }

  let { name, class: className = "", label }: Props = $props();
</script>

<i
  class="lni lni-{name} {className}"
  aria-hidden={label ? undefined : true}
  aria-label={label}
  role={label ? "img" : undefined}
></i>
