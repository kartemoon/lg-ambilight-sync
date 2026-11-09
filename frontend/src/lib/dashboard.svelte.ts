import { request, SettingsWriter } from './api';
import type { LiveSettings } from './settings';
import type { PreviewStatus } from './preview-types';
import type { DiagnosticStatus } from './diagnostics-types';
export interface MusicOutput {
  sensitivity: number;
  intensity: number;
}
export interface MusicSettings {
  enabled: boolean;
  strip: MusicOutput;
  lamp: MusicOutput;
  top_intensity?: number;
}
export interface Status extends PreviewStatus, DiagnosticStatus {
  music?: MusicSettings;
  music_input_active?: boolean;
  service_instance: number;
  settings: LiveSettings;
  active_preset: string | null;
  preset_modified: boolean;
  bonjour_enabled: boolean;
  hue_bridge_ip: string;
  nanoleaf_ip: string;
  calibration_pattern?: string | null;
}
export class Dashboard {
  status = $state.raw<Status | null>(null);
  settings = $state.raw<LiveSettings | null>(null);
  receivedAt = $state(0);
  display = $state<boolean | null>(null);
  busy = $state(false);
  saving = $state(false);
  error = $state('');
  notice = $state('');
  serviceRestartMessage = $state('');
  private restartingService = false;
  dirty = $state(false);
  online = $state(false);
  private timer: ReturnType<typeof setTimeout> | undefined;
  private saveTimer: ReturnType<typeof setTimeout> | undefined;
  private stopped = false;
  private polling = false;
  private revision = 0;
  private writer = new SettingsWriter<LiveSettings>(async (value) => {
    await request('/api/settings', value);
  });
  async refresh() {
    if (this.polling || this.stopped) return;
    this.polling = true;
    const revision = this.revision;
    try {
      const status = await request<Status>('/api/status');
      if (!status.settings || typeof status.pipeline_state !== 'string')
        throw new Error('Некорректный ответ от телевизора');
      this.status = status;
      this.receivedAt = Date.now();
      this.online = true;
      if (!this.writer.pending && !this.busy && !this.dirty && revision === this.revision)
        this.settings = status.settings;
    } catch (error) {
      this.online = false;
      if (!this.restartingService) this.error = message(error);
    } finally {
      this.polling = false;
    }
  }
  async refreshDisplay() {
    try {
      const data = await request<{ enabled: unknown }>('/api/display');
      this.display = typeof data.enabled === 'boolean' ? data.enabled : null;
    } catch {
      this.display = null;
    }
  }
  start() {
    this.stopped = false;
    void this.loop();
    return () => this.stop();
  }
  private async loop() {
    if (this.stopped) return;
    if (!document.hidden) await Promise.all([this.refresh(), this.refreshDisplay()]);
    if (!this.stopped) this.timer = setTimeout(() => void this.loop(), 1500);
  }
  stop() {
    this.stopped = true;
    clearTimeout(this.timer);
    clearTimeout(this.saveTimer);
  }
  patch(patch: Partial<LiveSettings>) {
    if (!this.settings || this.busy) return;
    this.revision++;
    this.settings = { ...this.settings, ...patch };
    this.dirty = true;
    this.error = '';
    const revision = this.revision;
    void this.writer
      .push(structuredClone(this.settings))
      .then(() => {
        if (revision === this.revision) this.scheduleSave();
      })
      .catch((error) => {
        this.error = message(error);
      });
  }
  private scheduleSave() {
    if (this.stopped) return;
    clearTimeout(this.saveTimer);
    this.saveTimer = setTimeout(() => void this.save(), 1000);
  }
  async save() {
    if (this.busy || this.saving) {
      this.scheduleSave();
      return;
    }
    clearTimeout(this.saveTimer);
    this.saving = true;
    const revision = this.revision;
    try {
      await this.writer.flush();
      await request('/api/save-config', undefined, 'POST');
      if (revision === this.revision) {
        this.dirty = false;
        this.notice = 'Настройки сохранены.';
      }
      await this.refresh();
    } catch (error) {
      this.error = 'Не удалось сохранить: ' + message(error);
    } finally {
      this.saving = false;
    }
  }
  async preset(name: string, reset = false) {
    if (this.busy || (!reset && this.status?.active_preset === name)) return;
    this.busy = true;
    this.error = '';
    try {
      await this.writer.flush();
      await request(
        `/api/presets/${encodeURIComponent(name)}${reset ? '/reset' : ''}`,
        undefined,
        'POST',
      );
      this.revision++;
      const status = await request<Status>('/api/status');
      this.status = status;
      this.settings = status.settings;
      this.receivedAt = Date.now();
      this.dirty = true;
      this.scheduleSave();
    } catch (error) {
      this.error = message(error);
    } finally {
      this.busy = false;
    }
  }
  async command<T = unknown>(path: string, body?: unknown): Promise<T | undefined> {
    if (this.busy) return;
    this.busy = true;
    this.error = '';
    try {
      await this.writer.flush();
      const result = await request<T>(path, body, 'POST');
      this.notice = 'Применено.';
      return result;
    } catch (error) {
      this.error = message(error);
      throw error;
    } finally {
      this.busy = false;
      await Promise.all([this.refresh(), this.refreshDisplay()]);
    }
  }
  async restartService() {
    if (this.busy || this.saving) return;
    const warning = this.dirty ? ' Несохранённые изменения могут потеряться.' : '';
    if (!confirm('Полностью перезапустить службу подсветки?' + warning)) return;

    this.busy = true;
    this.restartingService = true;
    this.error = '';
    this.serviceRestartMessage = 'Перезапускаю службу подсветки…';
    try {
      const { service_instance: previous } = await request<{ service_instance: number }>(
        '/api/restart-service',
        {},
      );
      const deadline = Date.now() + 70_000;
      while (!this.stopped && Date.now() < deadline) {
        await new Promise((resolve) => setTimeout(resolve, 1000));
        await this.refresh();
        if (this.online && this.status?.service_instance && this.status.service_instance !== previous) {
          this.settings = this.status.settings;
          this.dirty = false;
          clearTimeout(this.saveTimer);
          this.serviceRestartMessage = 'Служба подсветки перезапущена.';
          return;
        }
      }
      throw new Error('Служба не ответила после перезапуска.');
    } catch (error) {
      this.serviceRestartMessage = '';
      this.error = message(error);
    } finally {
      this.restartingService = false;
      this.busy = false;
    }
  }
  get requested() {
    return (
      !!this.status &&
      (this.status.pipeline_state === 'running' ||
        (!['paused', 'faulted'].includes(this.status.pipeline_state) &&
          this.status.requested_running === true))
    );
  }
}
export function message(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
