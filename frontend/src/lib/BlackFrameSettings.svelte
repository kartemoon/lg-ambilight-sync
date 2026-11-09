<script lang="ts">
  import Fader from './Fader.svelte';
  import type { LiveSettings } from './settings';

  let {
    settings,
    disabled = false,
    onchange,
  }: {
    settings: LiveSettings | null;
    disabled?: boolean;
    onchange: (patch: Partial<LiveSettings>) => void;
  } = $props();

  const holdHelp =
    'Ненадолго сохранять последние цвета, когда весь экран темнеет. Ноль отключает задержку.';
  const fadeHelp =
    'Плавно погасить подсветку после задержки. При нуле подсветка погаснет сразу.';
  const seconds = (value: number) => `${Number((value / 1000).toFixed(2))} с`;
</script>

<fieldset class="black-frame-settings">
  <legend>Поведение при чёрном экране</legend>
  <p class="muted">
    Когда весь экран темнеет, подсветка сохраняет последний цвет, а затем гаснет. Если картинка вернётся, затухание отменится. Чтобы отключить функцию, выставь оба значения в ноль.
  </p>
  <Fader
    label="Задержка перед затуханием"
    value={settings?.black_frame_hold_ms ?? 750}
    stops={[0, 250, 500, 750, 1000, 1500, 2000, 3000, 5000]}
    format={seconds}
    help={holdHelp}
    disabled={!settings || disabled}
    onchange={(value) => onchange({ black_frame_hold_ms: value })}
    onhelp={() => {}}
    vertical={false}
  />
  <Fader
    label="Время затухания"
    value={settings?.black_frame_fade_ms ?? 2000}
    stops={[0, 250, 500, 1000, 1500, 2000, 3000, 5000, 10000]}
    format={seconds}
    help={fadeHelp}
    disabled={!settings || disabled}
    onchange={(value) => onchange({ black_frame_fade_ms: value })}
    onhelp={() => {}}
    vertical={false}
  />
</fieldset>

<style>
  .black-frame-settings {
    display: grid;
    gap: 1rem;
    min-inline-size: 0;
    padding: 0;
    border: 0;
  }

  legend {
    font-weight: 600;
    margin-block-end: 0.5rem;
  }
</style>
