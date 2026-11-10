export interface LiveSettings {
  brightness_multiplier: number;
  hue_output_brightness: number;
  nanoleaf_output_brightness: number;
  saturation_boost: number;
  peak_weight: number;
  spatial_blend: number;
  depth_influence: number;
  gamma: number;
  noise_gate_threshold: number;
  smoothing_factor: number;
  rise_smoothing_factor: number;
  fall_smoothing_factor: number;
  strict_blackout: boolean;
  black_frame_hold_ms: number;
  black_frame_fade_ms: number;
  use_xy_gamut: boolean;
  letterbox_detection: boolean;
  hdmi_auto_fps: boolean | null;
  hdr_tone_mapping: boolean;
  hue_sync_enabled: boolean;
  nanoleaf_sync_enabled: boolean;
  auto_tv_power: boolean;
  capture_youtube: boolean;
  capture_kinopoisk: boolean;
  capture_lampa: boolean;
  capture_ivi: boolean;
  capture_reelo: boolean;
  capture_okko: boolean;
  nanoleaf_idle_enabled: boolean;
  nanoleaf_idle_scene: string;
  nanoleaf_idle_colour: string;
  nanoleaf_alignment: {
    start_corner: string;
    reverse_direction: boolean;
    perimeter_offset: number;
  };
  max_color_step: number;
}
export const initialSettings: LiveSettings = {
  brightness_multiplier: 1,
  hue_output_brightness: 1,
  nanoleaf_output_brightness: 1,
  saturation_boost: 1,
  peak_weight: 0.15,
  spatial_blend: 0.15,
  depth_influence: 0.5,
  gamma: 1,
  noise_gate_threshold: 0.02,
  smoothing_factor: 0.35,
  rise_smoothing_factor: 0.35,
  fall_smoothing_factor: 0.35,
  strict_blackout: false,
  black_frame_hold_ms: 750,
  black_frame_fade_ms: 2000,
  use_xy_gamut: false,
  letterbox_detection: true,
  hdmi_auto_fps: false,
  hdr_tone_mapping: false,
  hue_sync_enabled: true,
  nanoleaf_sync_enabled: true,
  auto_tv_power: true,
  capture_youtube: true,
  capture_kinopoisk: true,
  capture_lampa: true,
  capture_ivi: true,
  capture_reelo: true,
  capture_okko: true,
  nanoleaf_idle_enabled: false,
  nanoleaf_idle_scene: "",
  nanoleaf_idle_colour: "#604321",
  nanoleaf_alignment: {
    start_corner: 'bottom-center',
    reverse_direction: false,
    perimeter_offset: 0,
  },
  max_color_step: 12,
};
export type NumericKey =
  | 'brightness_multiplier'
  | 'saturation_boost'
  | 'peak_weight'
  | 'spatial_blend'
  | 'gamma'
  | 'smoothing_factor'
  | 'max_color_step'
  | 'noise_gate_threshold'
  | 'hue_output_brightness'
  | 'nanoleaf_output_brightness';
export const faders: {
  key: NumericKey;
  label: string;
  stops: number[];
  format: (v: number) => string;
  help: string;
}[] = [
  {
    key: 'brightness_multiplier',
    label: 'Яркость',
    stops: [0.5, 1, 1.5, 2, 2.5],
    format: (v) => `${Number(v.toFixed(2))}×`,
    help: 'Общая яркость подсветки. Яркость каждого устройства настраивается отдельно.',
  },
  {
    key: 'saturation_boost',
    label: 'Насыщенность',
    stops: [0.5, 1, 1.5, 2, 2.5],
    format: (v) => `${Number(v.toFixed(2))}×`,
    help: 'Ниже 1 цвета бледнее, выше 1 — насыщеннее. При 1 насыщенность не меняется.',
  },
  {
    key: 'peak_weight',
    label: 'Вес ярких участков',
    stops: [0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1],
    format: (v) => `${Math.round(v * 100)}%`,
    help: 'Даёт ярким участкам больший вес при выборе цвета подсветки. В предпросмотре эффект не виден.',
  },
  {
    key: 'spatial_blend',
    label: 'Смешивание',
    stops: [0, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45, 0.5],
    format: (v) => `${Math.round(v * 100)}%`,
    help: 'Смешивает цвета соседних зон. Чёрные зоны остаются выключенными.',
  },
  {
    key: 'gamma',
    label: 'Гамма',
    stops: [0.5, 0.6, 0.7, 0.85, 1, 1.2, 1.4, 1.7, 2],
    format: (v) => v.toFixed(2),
    help: 'Выше 1 средние тона темнее, ниже 1 — светлее. Нейтральное значение — 1.',
  },
  {
    key: 'smoothing_factor',
    label: 'Скорость отклика',
    stops: [0.05, 0.08, 0.12, 0.18, 0.25, 0.35, 0.5, 0.65, 0.8, 1],
    format: (v) => `${Math.round(v * 100)}%`,
    help: 'Чем выше значение, тем быстрее реакция. При 100% сглаживание отключено. Ползунок задаёт одинаковую скорость нарастания и спада.',
  },
  {
    key: 'max_color_step',
    label: 'Плавность смены сцен',
    stops: [4, 6, 8, 12, 16, 24, 32, 48, 64],
    format: (v) => `${v} steps`,
    help: 'Ограничивает резкие скачки цвета. Чем меньше значение, тем плавнее переходы.',
  },
  {
    key: 'noise_gate_threshold',
    label: 'Порог чёрного',
    stops: [0, 0.005, 0.01, 0.015, 0.02, 0.03, 0.04, 0.06],
    format: (v) => `${Number((v * 100).toFixed(1))}%`,
    help: 'Считает очень тёмные участки чёрными. Режим гашения полностью выключает соответствующие зоны.',
  },
];
export const colourModes = [
  {
    value: 'softColor',
    label: 'Мягкий',
    help: 'Меньше насыщенности и яркости, мягкие блики.',
  },
  {
    value: 'neutral',
    label: 'Нейтральный',
    help: 'Нейтральные цвета, яркость и гамма.',
  },
  {
    value: 'vividColor',
    label: 'Насыщенный',
    help: 'Более насыщенные цвета, яркие блики и глубокий контраст.',
  },
];
export const responseModes = {
  low: [0.2, 0.18, 0.22, 6],
  default: [0.35, 0.35, 0.35, 12],
  fast: [0.65, 0.65, 0.7, 14],
} as const;
export const blendModes = { local: 0, balanced: 0.15, soft: 0.3 } as const;
export function nearestStop(value: number, stops: readonly number[]) {
  return stops.reduce(
    (best, stop, i) => (Math.abs(stop - value) < Math.abs(stops[best] - value) ? i : best),
    0,
  );
}
export function responseMode(settings: LiveSettings | null) {
  return (
    Object.entries(responseModes).find(
      ([, v]) =>
        settings &&
        [
          settings.smoothing_factor,
          settings.rise_smoothing_factor,
          settings.fall_smoothing_factor,
          settings.max_color_step,
        ].every((n, i) => Math.abs(n - v[i]) < 0.00001),
    )?.[0] ?? ''
  );
}
export function blendMode(settings: LiveSettings | null) {
  return (
    Object.entries(blendModes).find(
      ([, v]) => settings && Math.abs(settings.spatial_blend - v) < 0.00001,
    )?.[0] ?? ''
  );
}
