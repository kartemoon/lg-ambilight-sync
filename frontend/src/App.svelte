<script lang="ts">
  import { onMount } from 'svelte';
  import { RefreshCw, Monitor, Moon, Sun, Circle, Save, RotateCcw } from '@lucide/svelte';
  import SettingToggle from './lib/SettingToggle.svelte';
  import { Toggle } from './lib/components/ui/toggle';
  import { Button } from './lib/components/ui/button';
  import { Dashboard } from './lib/dashboard.svelte';
  import {
    blendMode,
    blendModes,
    colourModes,
    faders,
    responseMode,
    responseModes,
    type LiveSettings,
  } from './lib/settings';
  import Segmented from './lib/Segmented.svelte';
  import Fader from './lib/Fader.svelte';
  import Preview from './lib/Preview.svelte';
  import Devices from './lib/Devices.svelte';
  import IdleLighting from './lib/IdleLighting.svelte';
  import MusicSync from './lib/MusicSync.svelte';
  import InstallApp from './lib/InstallApp.svelte';
  import BlackFrameSettings from './lib/BlackFrameSettings.svelte';
  import Diagnostics from './lib/Diagnostics.svelte';
  import { vibrate } from './lib/haptics';
  import { registerAgentTools } from './lib/agent-tools';
  const model = new Dashboard();
  let tab = $state('tuning'),
    theme = $state('system'),
    now = $state(Date.now());
  let help = $state({
    title: 'Управление откликом',
    text: 'Нажми на ползунок, чтобы узнать, что он меняет.',
  });
  let presetHelp = $state('Цвет, скорость отклика и смешивание зон настраиваются отдельно.');
  let settings = $derived(model.settings);
  const responseOptions = [
    { value: 'low', label: 'Плавный', help: 'Более плавные переходы при смене кадров.' },
    {
      value: 'default',
      label: 'Обычный',
      help: 'Баланс скорости и плавности.',
    },
    { value: 'fast', label: 'Быстрый', help: 'Подсветка быстрее реагирует на смену кадра.' },
  ];
  const blendOptions = [
    {
      value: 'local',
      label: 'Только своя зона',
      help: 'Каждый светильник учитывает только свою зону экрана.',
    },
    {
      value: 'balanced',
      label: 'Умеренное',
      help: 'Добавляет 15% цвета соседних зон для плавных переходов.',
    },
    {
      value: 'soft',
      label: 'Мягкое',
      help: 'Добавляет 30% цвета соседних зон для более ровной подсветки.',
    },
  ];
  const depthOptions = [
    {
      value: '0',
      label: 'Нет',
      help: 'Размер зоны экрана не зависит от глубины комнаты.',
    },
    {
      value: '0.5',
      label: 'Слабое',
      help: 'Немного расширяет зоны для задних светильников: до 31,25%.',
    },
    {
      value: '1',
      label: 'Сильное',
      help: 'Сильнее расширяет зоны для задних светильников: до 37,5%.',
    },
  ];
  let depthHelp = $state(
    'Меняет размер зон по глубине комнаты Hue. Положение каналов сохраняется.',
  );
  const tabs = [
    ['tuning', 'Настройка'],
    ['devices', 'Устройства'],
    ['diagnostics', 'Диагностика'],
  ];
  const themeNames: Record<string, string> = { system: 'системная', dark: 'тёмная', light: 'светлая' };
  const demo = import.meta.env.DEV;
  function changeResponse(value: string) {
    const [s, r, f, max] = responseModes[value as keyof typeof responseModes];
    model.patch({
      smoothing_factor: s,
      rise_smoothing_factor: r,
      fall_smoothing_factor: f,
      max_color_step: max,
    });
  }
  function changeBlend(value: string) {
    model.patch({ spatial_blend: blendModes[value as keyof typeof blendModes] });
  }
  function changeSetting(key: keyof LiveSettings, value: number) {
    model.patch(
      key === 'smoothing_factor'
        ? { smoothing_factor: value, rise_smoothing_factor: value, fall_smoothing_factor: value }
        : { [key]: value },
    );
  }
  function cycleTheme() {
    theme = ['system', 'dark', 'light'][(['system', 'dark', 'light'].indexOf(theme) + 1) % 3];
    applyTheme();
  }
  function applyTheme() {
    if (theme === 'system') delete document.documentElement.dataset.theme;
    else document.documentElement.dataset.theme = theme;
    try {
      localStorage.setItem('lg-hue-sync-theme', theme);
    } catch {
      /* Storage may be unavailable. */
    }
  }
  function tabKey(event: KeyboardEvent, index: number) {
    const direction = event.key === 'ArrowRight' ? 1 : event.key === 'ArrowLeft' ? -1 : 0;
    if (!direction && event.key !== 'Home' && event.key !== 'End') return;
    event.preventDefault();
    tab =
      tabs[
        event.key === 'Home'
          ? 0
          : event.key === 'End'
            ? tabs.length - 1
            : (index + direction + tabs.length) % tabs.length
      ][0];
    document.getElementById(`tab-${tab}`)?.focus();
  }
  onMount(() => {
    try {
      theme = localStorage.getItem('lg-hue-sync-theme') || 'system';
    } catch {
      /* Storage may be unavailable. */
    }
    applyTheme();
    const stop = model.start();
    const unregister = registerAgentTools(model);
    const clock = setInterval(() => (now = Date.now()), 1000);
    return () => {
      stop();
      unregister();
      clearInterval(clock);
    };
  });
</script>

<svelte:document
  onclick={(event) => {
    if (event.isTrusted && (event.target as Element).closest('button:not(:disabled), summary'))
      vibrate();
  }}
/>
<main>
  {#if demo}<p class="preview-badge">
      <span class="status-dot active"></span>Предпросмотр · ТВ и светильники имитируются
    </p>{/if}
  <header class="header-actions" aria-label="Управление экраном и синхронизацией">
    <Toggle
      variant="outline"
      class="power-button"
      bind:pressed={
        () => model.requested,
        (pressed) => model.command(pressed ? '/api/start' : '/api/stop').catch(() => {})
      }
      disabled={!model.status || model.busy}
      ><RefreshCw size={17} />Подсветка: {model.requested ? 'Вкл.' : 'Выкл.'}</Toggle
    >
    <Toggle
      variant="outline"
      class="power-button"
      bind:pressed={
        () => model.display === true, () => model.command('/api/display/toggle').catch(() => {})
      }
      disabled={model.display === null || model.busy}
      ><Monitor size={17} />Экран {model.display === null
        ? '—'
        : model.display
          ? 'Вкл.'
          : 'Выкл.'}</Toggle
    >
    <Button
      variant="outline"
      class="icon-button"
      aria-label={`Тема: ${themeNames[theme]}. Сменить оформление`}
      title={`Тема: ${themeNames[theme]}`}
      onclick={cycleTheme}
      >{#if theme === 'light'}<Sun />{:else if theme === 'dark'}<Moon />{:else}<Circle
        />{/if}</Button
    >
  </header>
  <Button
    variant="outline"
    disabled={model.busy || model.saving || !model.status}
    onclick={() => void model.restartService()}
  ><RotateCcw size={16} />Полный перезапуск подсветки</Button>
  {#if model.serviceRestartMessage}
    <div class="notice" role="status">{model.serviceRestartMessage}</div>
  {/if}
  {#if model.error}<div class="notice error" role="alert">{model.error}</div>{/if}
  <div class="dashboard-tabs" role="tablist" aria-label="Разделы панели">
    {#each tabs as [value, label], index (value)}<Button
        variant="ghost"
        id={`tab-${value}`}
        role="tab"
        aria-controls={`panel-${value}`}
        aria-selected={tab === value}
        tabindex={tab === value ? 0 : -1}
        onclick={() => (tab = value)}
        onkeydown={(event) => tabKey(event, index)}>{label}</Button
      >{/each}
  </div>
  <div id="panel-tuning" role="tabpanel" aria-labelledby="tab-tuning" hidden={tab !== 'tuning'}>
    <IdleLighting {model} />
    <MusicSync {model} />
    <section class="card section-body" aria-label="Частота захвата">
      <SettingToggle
        title="Авто для HDMI"
        description="Лимит 60 FPS на HDMI и 30 FPS в приложениях. Если выключено, везде 30 FPS."
        checked={settings?.hdmi_auto_fps ?? false}
        disabled={!settings || model.busy}
        onchange={(checked) => model.patch({ hdmi_auto_fps: checked })}
      />
    </section>
    <section class="card preset-panel">
      <h2>Стиль подсветки</h2>
      <Segmented
        label="Цвет"
        options={colourModes}
        value={model.status?.active_preset ?? ''}
        disabled={!settings || model.busy}
        onchange={(value) => model.preset(value)}
        onhelp={(text) => (presetHelp = text)}
      />
      <Segmented
        label="Скорость отклика"
        options={responseOptions}
        value={responseMode(settings)}
        disabled={!settings || model.busy}
        onchange={changeResponse}
        onhelp={(text) => (presetHelp = text)}
      />
      <Segmented
        label="Смешивание зон"
        options={blendOptions}
        value={blendMode(settings)}
        disabled={!settings || model.busy}
        onchange={changeBlend}
        onhelp={(text) => (presetHelp = text)}
      />
      <p class="preset-help muted">{presetHelp}</p>
      <p class="save-state" role="status" aria-live="polite">
        {model.saving
          ? 'Сохраняю…'
          : model.dirty
            ? 'Сохраняю изменения…'
            : model.online
              ? 'Изменения сохраняются автоматически.'
              : 'Подключение к телевизору…'}
      </p>
    </section>
    <Preview status={model.status} {settings} active={tab === 'tuning'} />
    <section class="card tuning-panel">
      <div class="panel-heading">
        <h2>Точная настройка</h2>
        <div class="button-row">
          <Button
            variant="outline"
            class="icon-button"
            aria-label="Сохранить настройки"
            title="Сохранить сейчас"
            disabled={!settings || model.busy || model.saving}
            onclick={() => model.save()}><Save /></Button
          ><Button
            variant="outline"
            class="icon-button"
            aria-label="Сбросить настройки цвета"
            title="Сбросить цвет, сохранив отклик и смешивание"
            disabled={!model.status?.active_preset || model.busy}
            onclick={() => {
              if (model.status?.active_preset) void model.preset(model.status.active_preset, true);
            }}><RotateCcw /></Button
          >
        </div>
      </div>
      <div class="fader-bank" role="group" aria-label="Настройки цвета и отклика">
        {#each faders as fader (fader.key)}<div class="fader-card">
            <Fader
              label={fader.label}
              value={settings?.[fader.key] ?? fader.stops[0]}
              stops={fader.stops}
              format={fader.format}
              help={fader.help}
              disabled={!settings || model.busy}
              onchange={(value) => changeSetting(fader.key, value)}
              onhelp={() => (help = { title: fader.label, text: fader.help })}
            />{#if fader.key === 'noise_gate_threshold'}<SettingToggle
                compact
                title="Гасить тёмные зоны"
                checked={settings?.strict_blackout ?? false}
                disabled={!settings || model.busy}
                onchange={(checked) => model.patch({ strict_blackout: checked })}
              />{/if}
          </div>{/each}
      </div>
      <aside class="tuning-help" aria-live="polite">
        <strong>{help.title}</strong>
        <p>{help.text}</p>
      </aside>
      <details class="processing">
        <summary>Обработка изображения</summary>
        <div class="section-body">
          <BlackFrameSettings
            {settings}
            disabled={model.busy}
            onchange={(patch) => model.patch(patch)}
          />
          <Segmented
            label="Влияние глубины комнаты"
            options={depthOptions}
            value={String(settings?.depth_influence ?? 0.5)}
            disabled={!settings || model.busy}
            onchange={(value) => model.patch({ depth_influence: Number(value) })}
            onhelp={(text) => (depthHelp = text)}
          />
          <p class="muted">{depthHelp}</p>
          <SettingToggle
            title="Прямой режим RGB"
            description="Повышает яркость Hue. Если выключено, используется CIE 1931 xy."
            checked={settings ? !settings.use_xy_gamut : false}
            disabled={!settings || model.busy}
            onchange={(checked) => model.patch({ use_xy_gamut: !checked })}
          />

          <SettingToggle
            title="Игнорировать чёрные полосы"
            description="Считывать цвета только из области видео между чёрными полосами."
            checked={settings?.letterbox_detection ?? false}
            disabled={!settings || model.busy}
            onchange={(checked) => model.patch({ letterbox_detection: checked })}
          />

          <SettingToggle
            title="Осветление средних тонов"
            description="Осветляет средние тона. Не корректирует HDR."
            checked={settings?.hdr_tone_mapping ?? false}
            disabled={!settings || model.busy}
            onchange={(checked) => model.patch({ hdr_tone_mapping: checked })}
          />
        </div>
      </details>
    </section>
  </div>
  <div id="panel-devices" role="tabpanel" aria-labelledby="tab-devices" hidden={tab !== 'devices'}>
    <Devices {model} />
    <InstallApp />
  </div>
  <div
    id="panel-diagnostics"
    role="tabpanel"
    aria-labelledby="tab-diagnostics"
    hidden={tab !== 'diagnostics'}
  >
    <Diagnostics status={model.status} receivedAt={model.receivedAt} online={model.online} {now} />
  </div>
</main>
