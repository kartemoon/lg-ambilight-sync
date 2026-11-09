<script lang="ts">
  import SettingToggle from './SettingToggle.svelte';
  import { Button } from './components/ui/button';
  import { request } from './api';
  import type { Dashboard } from './dashboard.svelte';

  let { model }: { model: Dashboard } = $props();
  let settings = $derived(model.settings);
  let scenes = $state<string[]>([]);
  let loading = $state(false);
  let error = $state('');
  let previousController = '';
  async function loadScenes() {
    loading = true;
    error = '';
    try {
      scenes = await request<string[]>('/api/nanoleaf/scenes');
    } catch {
      error = 'Не удалось получить сцены Nanoleaf. Постоянный цвет по-прежнему доступен.';
    } finally {
      loading = false;
    }
  }
  $effect(() => {
    const controller = model.status?.nanoleaf_ip ?? '';
    if (controller && controller !== previousController) {
      previousController = controller;
      void loadScenes();
    }
  });
</script>

<section class="card section-body" aria-label="Фоновая подсветка Nanoleaf">
  <SettingToggle
    title="Фоновая подсветка Nanoleaf"
    description="Включается вместе с ТВ, пока захват экрана на паузе. При запуске захвата подсветка переключается на изображение."
    checked={settings?.nanoleaf_idle_enabled ?? false}
    disabled={!settings || model.busy}
    onchange={(checked) => model.patch({ nanoleaf_idle_enabled: checked })}
  />
  {#if settings?.nanoleaf_idle_enabled}
    <label class="field">
      Цвет или сцена без захвата
      <select
        value={settings.nanoleaf_idle_scene}
        disabled={model.busy}
        onchange={(e) => model.patch({ nanoleaf_idle_scene: e.currentTarget.value })}
      >
        <option value="">Постоянный цвет</option>
        {#if settings.nanoleaf_idle_scene && !scenes.includes(settings.nanoleaf_idle_scene)}
          <option value={settings.nanoleaf_idle_scene}>{settings.nanoleaf_idle_scene}</option>
        {/if}
        {#each scenes as scene (scene)}<option value={scene}>{scene}</option>{/each}
      </select>
    </label>
    {#if !settings.nanoleaf_idle_scene}
      <label class="field">Постоянный цвет
        <input type="color" value={settings.nanoleaf_idle_colour}
          disabled={model.busy}
          onchange={(e) => model.patch({ nanoleaf_idle_colour: e.currentTarget.value })} />
      </label>
    {/if}
    <Button variant="secondary" disabled={loading || !model.status?.nanoleaf_ip} onclick={loadScenes}>
      {loading ? 'Загружаю сцены…' : 'Обновить список сцен'}
    </Button>
    {#if error}<p class="muted" role="status">{error}</p>{/if}
    {#if !settings.auto_tv_power}<p class="muted">Для включения вместе с ТВ включи «Следовать питанию ТВ» в разделе «Устройства».</p>{/if}
  {/if}
</section>
