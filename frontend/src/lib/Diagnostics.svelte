<script lang="ts">
  import CaptureLightingGauge from './CaptureLightingGauge.svelte';
  import {
    diagnosticRate,
    diagnosticValue,
    snapshotState,
    type DiagnosticStatus,
  } from './diagnostics-types';

  let {
    status,
    receivedAt = 0,
    now = Date.now(),
    online = true,
  }: {
    status: DiagnosticStatus | null;
    receivedAt?: number;
    now?: number;
    online?: boolean;
  } = $props();

  const unavailable = 'Unavailable';
  const size = (width?: number | null, height?: number | null) =>
    width != null && height != null ? `${width} × ${height}` : 'Нет данных';
  const dynamicRange = (hdr?: string | null) => (hdr === 'none' ? 'SDR' : diagnosticValue(hdr));

  let snapshot = $derived(status?.diagnostics);
  let freshness = $derived(snapshotState(snapshot, now));
  let snapshotFresh = $derived(freshness.available && !freshness.stale);
  let display = $derived(
    snapshotFresh && snapshot?.display?.available ? snapshot.display : undefined,
  );
  let video = $derived(snapshotFresh && snapshot?.video?.available ? snapshot.video : undefined);
  let player = $derived(snapshotFresh && snapshot?.player?.available ? snapshot.player : undefined);
  let output = $derived(snapshotFresh && snapshot?.output?.available ? snapshot.output : undefined);
  let freshnessMessage = $derived(
    !freshness.available
      ? 'Нет данных о сигнале.'
      : freshness.stale
        ? 'Данные устарели. Ожидание новых показаний.'
        : `Данные о сигнале обновлены ${Math.floor(freshness.age / 1000)} с назад.`,
  );

  const sectionState = (available: boolean) =>
    freshness.stale ? 'Stale' : available ? 'Available' : unavailable;
  let cards = $derived([
    {
      id: 'display',
      title: 'Выход на экран',
      state: sectionState(!!display),
      rows: [
        ['Частота обновления экрана', diagnosticRate(display?.refresh_rate)],
        ['Размер видео после масштабирования', size(display?.width, display?.height)],
        ['Источник видео', diagnosticValue(display?.source)],
        ['Экран включён', diagnosticValue(display?.enabled)],
        ['Изображение скрыто', diagnosticValue(display?.muted)],
        ['Синхронизация сигнала', diagnosticValue(display?.locked)],
      ],
      note: 'Частоту обновления сообщает драйвер экрана. Она может отличаться от частоты кадров видео. Размер видео указан после масштабирования.',
    },
    {
      id: 'video',
      title: 'Видео HDMI',
      state: sectionState(!!video),
      rows: [
        ['Вход', diagnosticValue(video?.input)],
        ['Разрешение', size(video?.width, video?.height)],
        ['Частота кадров', diagnosticRate(video?.frame_rate)],
        ['Развёртка', diagnosticValue(video?.scan_type)],
        ['Динамический диапазон', dynamicRange(video?.hdr)],
        ['Формат пикселей', diagnosticValue(video?.pixel_encoding)],
        ['Диапазон сигнала', diagnosticValue(video?.range)],
        ['Цветовое пространство', diagnosticValue(video?.colourimetry)],
        ['Глубина цвета', diagnosticValue(video?.bit_depth, ' бит')],
      ],
    },
    {
      id: 'player',
      title: 'Встроенный плеер',
      state: sectionState(!!player),
      rows: [
        ['Разрешение видео', size(player?.width, player?.height)],
        ['Частота кадров видео', diagnosticRate(player?.frame_rate)],
        ['Развёртка', diagnosticValue(player?.scan_type)],
        ['Динамический диапазон', dynamicRange(player?.hdr)],
      ],
      note: 'Данные появляются, когда приложение использует видеодекодер ТВ.',
    },
    {
      id: 'output',
      title: 'eARC / аудиовыход',
      state: sectionState(!!output),
      rows: [
        ['Возвратный аудиоканал', diagnosticValue(output?.arc_state)],
        ['Сквозная передача', diagnosticValue(output?.passthrough)],
      ],
      note: 'Это состояние аудиовыхода ТВ.',
    },
  ]);
</script>

<CaptureLightingGauge {status} {receivedAt} {now} {online} />
<div class="diagnostics-heading">
  <h2>Диагностика входов и выходов</h2>
  <p>{freshnessMessage}</p>
</div>
<div class="diagnostics-grid">
  {#each cards as card (card.id)}
    <section class="card diagnostic-card" aria-labelledby={`diagnostic-${card.id}`}>
      <div class="diagnostic-card-header">
        <h3 id={`diagnostic-${card.id}`}>{card.title}</h3>
        <span
          class="diagnostic-state"
          data-state={card.state === 'Available' || card.state === 'Live'
            ? 'available'
            : card.state.toLowerCase()}>{({Available: 'Доступно', Live: 'Активно', Stale: 'Устарело', Unavailable: 'Нет данных'} as Record<string, string>)[card.state] ?? card.state}</span
        >
      </div>
      <dl class="diagnostic-list">
        {#each card.rows as [label, value] (label)}
          <div>
            <dt>{label}</dt>
            <dd>{value}</dd>
          </div>
        {/each}
      </dl>
      {#if card.note}<p class="diagnostic-note">{card.note}</p>{/if}
    </section>
  {/each}
</div>

<style>
  .diagnostic-list dd {
    font-variant-numeric: tabular-nums;
  }
</style>
