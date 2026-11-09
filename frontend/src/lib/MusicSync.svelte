<script lang="ts">
  import SettingToggle from './SettingToggle.svelte';
  import type { Dashboard, MusicOutput, MusicSettings } from './dashboard.svelte';

  let { model }: { model: Dashboard } = $props();
  let music = $derived(model.status?.music);
  let pending = $state(false);
  const outputs = ['strip', 'lamp'] as const;

  async function apply(value: MusicSettings) {
    pending = true;
    try {
      await model.command('/api/music', value);
    } catch {
      /* The dashboard displays the request error. */
    } finally {
      pending = false;
    }
  }

  function adjust(output: 'strip' | 'lamp', field: keyof MusicOutput, value: number) {
    if (music) void apply({ ...music, [output]: { ...music[output], [field]: value } });
  }
</script>

{#if music}
  <section class="card section-body" aria-label="Музыкальная синхронизация">
    <SettingToggle
      title="Вспышки в такт музыке"
      description="Добавляет короткие плавные импульсы яркости к цветам экрана."
      checked={music.enabled}
      disabled={model.busy || pending}
      onchange={(enabled) => { if (music) void apply({ ...music, enabled }); }}
    />
    {#if music.enabled}
      <p class="muted">{model.status?.music_input_active ? 'Аудиосигнал поступает' : 'Ожидание аудиосигнала'}</p>
      {#each outputs as output}
        <fieldset disabled={model.busy || pending}>
          <legend>{output === 'strip' ? 'Лента ТВ' : 'Кольцо торшера'}</legend>
          <label class="field">
            Чувствительность · {music[output].sensitivity}%
            <input type="range" min="0" max="100" value={music[output].sensitivity}
              onchange={(e) => adjust(output, 'sensitivity', e.currentTarget.valueAsNumber)} />
          </label>
          <label class="field">
            Сила вспышки · {music[output].intensity}%
            <input type="range" min="0" max="100" value={music[output].intensity}
              onchange={(e) => adjust(output, 'intensity', e.currentTarget.valueAsNumber)} />
          </label>
        </fieldset>
      {/each}
      <fieldset disabled={model.busy || pending}>
        <legend>Верхний свет торшера</legend>
        <label class="field">
          Сила вспышки · {music.top_intensity ?? 100}%
          <input type="range" min="0" max="100" value={music.top_intensity ?? 100}
            onchange={(e) => {
              if (music) void apply({ ...music, top_intensity: e.currentTarget.valueAsNumber });
            }} />
        </label>
      </fieldset>
    {/if}
  </section>
{/if}
