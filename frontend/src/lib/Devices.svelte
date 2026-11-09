<script lang="ts">
  import SettingToggle from './SettingToggle.svelte';
  import { Button } from './components/ui/button';
  import { Link } from '@lucide/svelte';
  import Fader from './Fader.svelte';
  import type { Dashboard } from './dashboard.svelte';
  let { model }: { model: Dashboard } = $props();
  let hueIp = $state(''),
    nanoleafIp = $state(''),
    area = $state('');
  let areas = $state<{ id: string; name: string }[]>([]),
    areaMessage = $state(''),
    loaded = $state(false);
  let alignmentDraft = $state<{
    start_corner: string;
    reverse_direction: boolean;
    perimeter_offset: number;
  } | null>(null);
  let settings = $derived(model.settings);
  let alignment = $derived(
    alignmentDraft ??
      settings?.nanoleaf_alignment ?? {
        start_corner: 'bottom-center',
        reverse_direction: false,
        perimeter_offset: 0,
      },
  );
  async function run(path: string, body?: unknown) {
    try {
      await model.command(path, body);
    } catch {
      /* Shared error banner reports failures. */
    }
  }
  async function loadAreas() {
    try {
      const { request } = await import('./api');
      const result = await request<{
        areas: { id: string; name: string }[];
        selected_area_id: string;
      }>('/api/hue/areas');
      areas = result.areas;
      area = result.selected_area_id;
      areaMessage = `Найдено зон развлечений: ${areas.length}.`;
    } catch (error) {
      areaMessage = String(error);
    }
  }
  function showDevices(event: Event) {
    if ((event.currentTarget as HTMLDetailsElement).open && !loaded) {
      loaded = true;
      void loadAreas();
    }
  }
  async function saveAlignment() {
    try {
      await model.command('/api/nanoleaf/alignment', alignment);
      if (model.settings)
        model.settings = { ...model.settings, nanoleaf_alignment: { ...alignment } };
      alignmentDraft = null;
    } catch {
      /* Shared error banner reports failures. */
    }
  }
  const patterns = [
    ['red', 'Красный'],
    ['green', 'Зелёный'],
    ['blue', 'Синий'],
    ['white', 'Белый'],
    ['warm-white', 'Тёплый белый · 3000 К'],
    ['daylight-white', 'Дневной белый D65'],
    ['quadrants', 'Четыре цветовые зоны'],
    ['perimeter', 'Бегущий сегмент 4D'],
    ['off', 'Вернуться к синхронизации'],
  ];
  const cornerNames: Record<string, string> = { 'bottom-center': 'Снизу по центру', 'bottom-left': 'Снизу слева', 'top-left': 'Сверху слева', 'top-right': 'Сверху справа', 'bottom-right': 'Снизу справа' };
  const trimStops = [0, 0.5, 1, 1.5, 2, 2.5];
</script>

<details class="card accordion" ontoggle={showDevices} id="devicesSection">
  <summary>Устройства <span class="muted">Hue &amp; Nanoleaf</span></summary>
  <div class="section-body">
    <SettingToggle
      title="Следовать питанию ТВ"
      description="Приостанавливать подсветку в режиме ожидания и возобновлять при включении ТВ."
      checked={settings?.auto_tv_power ?? true}
      disabled={!settings || model.busy}
      onchange={(checked) => model.patch({ auto_tv_power: checked })}
    />
    <section class="device-subsection">
      <h2>Захват в приложениях</h2>
      <p class="muted">Выключенный источник ставит синхронизацию на паузу.</p>
      <SettingToggle
        title="YouTube"
        checked={settings?.capture_youtube ?? true}
        disabled={!settings || model.busy}
        onchange={(checked) => model.patch({ capture_youtube: checked })}
      />
      <SettingToggle
        title="Кинопоиск"
        checked={settings?.capture_kinopoisk ?? true}
        disabled={!settings || model.busy}
        onchange={(checked) => model.patch({ capture_kinopoisk: checked })}
      />
      <SettingToggle
        title="Lampa"
        checked={settings?.capture_lampa ?? true}
        disabled={!settings || model.busy}
        onchange={(checked) => model.patch({ capture_lampa: checked })}
      />
      <SettingToggle
        title="Иви"
        checked={settings?.capture_ivi ?? true}
        disabled={!settings || model.busy}
        onchange={(checked) => model.patch({ capture_ivi: checked })}
      />
    </section>
    <section class="device-subsection">
      <h2>Philips Hue</h2>
      <p class="muted">
        Нажми кнопку на Hue Bridge, затем запусти сопряжение в течение 45 секунд.
      </p>
      <label class="field"
        >Адрес Hue Bridge <input
          bind:value={hueIp}
          placeholder="Оставь пустым для поиска"
          inputmode="decimal"
        /></label
      >
      <Button
        variant="secondary"
        disabled={model.busy}
        onclick={() => run('/api/pair/hue', { ip: hueIp.trim() || null })}
        ><Link aria-hidden="true" />Подключить Hue Bridge</Button
      >
      <p class="device-status">
        {model.status?.hue_connected ? 'Подключено' : 'Нет связи'}{model.status?.hue_bridge_ip
          ? ` · ${model.status.hue_bridge_ip}`
          : ''}
      </p>
      <label class="field"
        >Зона развлечений Hue <select
          bind:value={area}
          disabled={!areas.length || model.busy}
          onchange={() => run('/api/hue/area', { area_id: area })}
          >{#each areas as item (item.id)}<option value={item.id}>{item.name}</option
            >{/each}</select
        ></label
      >
      <div class="button-row">
        <Button variant="secondary" onclick={loadAreas}>Обновить список зон</Button><Button
          variant="secondary"
          disabled={model.busy}
          onclick={() => run('/api/sync-bridge')}>Обновить схему светильников</Button
        >
      </div>
      <p class="muted" role="status">{areaMessage}</p>
      <SettingToggle
        title="Синхронизация Hue"
        description="Вернуть управление светильниками приложению Hue. Nanoleaf продолжит работать."
        checked={settings?.hue_sync_enabled ?? false}
        disabled={!settings || model.busy}
        onchange={(checked) => model.patch({ hue_sync_enabled: checked })}
      />
      {#if settings}<Fader
          label="Яркость Hue"
          value={settings.hue_output_brightness}
          stops={trimStops}
          format={(v) => `${v.toFixed(2)}×`}
          help="Меняет яркость только светильников Hue."
          vertical={false}
          onchange={(v) => model.patch({ hue_output_brightness: v })}
          onhelp={() => {}}
          disabled={model.busy}
        />{/if}
    </section>
    <section class="device-subsection">
      <h2>Nanoleaf 4D</h2>
      <p class="muted">Удерживай кнопку питания 5–7 секунд до мигания индикатора, затем запусти сопряжение.</p>
      <label class="field"
        >Адрес контроллера <input
          bind:value={nanoleafIp}
          placeholder={model.status?.nanoleaf_ip || 'IP-адрес Nanoleaf'}
          inputmode="decimal"
        /></label
      >
      <Button
        variant="secondary"
        disabled={model.busy}
        onclick={() => run('/api/pair/nanoleaf', { ip: nanoleafIp.trim() || null })}
        >Подключить Nanoleaf 4D</Button
      >
      <p class="device-status">
        {model.status?.nanoleaf_connected ? 'Подключено' : 'Нет связи'}{model.status?.nanoleaf_ip
          ? ` · ${model.status.nanoleaf_ip}`
          : ''}
      </p>
      <SettingToggle
        title="Синхронизация Nanoleaf 4D"
        description="Приостановить подсветку Nanoleaf. Hue продолжит работать."
        checked={settings?.nanoleaf_sync_enabled ?? false}
        disabled={!settings || model.busy}
        onchange={(checked) => model.patch({ nanoleaf_sync_enabled: checked })}
      />
      {#if settings}<Fader
          label="Яркость Nanoleaf"
          value={settings.nanoleaf_output_brightness}
          stops={trimStops}
          format={(v) => `${v.toFixed(2)}×`}
          help="Меняет яркость ленты без изменения захвата."
          vertical={false}
          onchange={(v) => model.patch({ nanoleaf_output_brightness: v })}
          onhelp={() => {}}
          disabled={model.busy}
        />{/if}
    </section>
  </div>
</details>
<details class="card accordion">
  <summary>Калибровка</summary>
  <div class="section-body">
    <p class="muted">
      Тестовые цвета помогают проверить совпадение сторон. Настройки изображения ТВ не меняются.
    </p>
    <a class="text-link" href="/capture-patterns" target="_blank" rel="noopener"
      >Открыть тестовые изображения ↗</a
    >
    <div class="button-row">
      {#each patterns as [value, label] (value)}<Button
          variant="secondary"
          disabled={model.busy}
          onclick={() => run(`/api/test-pattern/${value}`)}>{label}</Button
        >{/each}
    </div>
    <section class="device-subsection">
      <h2>Расположение ленты Nanoleaf</h2>
      <p class="muted">Запусти «Бегущий сегмент 4D», чтобы найти начало ленты.</p>
      <label class="field"
        >Начало ленты <select
          value={alignment.start_corner}
          onchange={(event) =>
            (alignmentDraft = { ...alignment, start_corner: event.currentTarget.value })}
          >{#each ['bottom-center', 'bottom-left', 'top-left', 'top-right', 'bottom-right'] as corner (corner)}<option
              value={corner}>{cornerNames[corner]}</option
            >{/each}</select
        ></label
      >
      <SettingToggle
        title="Обратное направление"
        description="Поменять направление обхода периметра ленты."
        checked={alignment.reverse_direction}
        onchange={(checked) => (alignmentDraft = { ...alignment, reverse_direction: checked })}
      />
      <label class="field"
        >Смещение по периметру · {alignment.perimeter_offset}<input
          type="range"
          min="0"
          max="39"
          step="1"
          value={alignment.perimeter_offset}
          oninput={(event) =>
            (alignmentDraft = {
              ...alignment,
              perimeter_offset: Number(event.currentTarget.value),
            })}
        /></label
      >
      <Button variant="secondary" disabled={model.busy} onclick={saveAlignment}
        >Применить расположение</Button
      >
    </section>
  </div>
</details>
<details class="card accordion">
  <summary>Обслуживание</summary>
  <div class="section-body">
    <SettingToggle
      title="Обнаружение через Bonjour"
      description="Показывать панель и API в локальной сети. Перезапуск не нужен."
      checked={model.status?.bonjour_enabled ?? false}
      disabled={!model.status || model.busy}
      onchange={(checked) => run('/api/bonjour', { enabled: checked })}
    />
    <Button
      variant="secondary"
      disabled={model.busy}
      onclick={() => {
        if (confirm('Перезапустить захват и подсветку? Подсветка ненадолго остановится.'))
          void run('/api/restart');
      }}>Перезапустить захват</Button
    >
  </div>
</details>
