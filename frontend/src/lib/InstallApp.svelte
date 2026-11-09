<script lang="ts">
  import { onMount } from 'svelte';
  import { Download } from '@lucide/svelte';
  import { Button } from './components/ui/button';

  interface InstallPrompt extends Event {
    prompt(): Promise<void>;
    userChoice: Promise<{ outcome: 'accepted' | 'dismissed' }>;
  }
  let prompt = $state<InstallPrompt | null>(null);
  let installed = $state(false);
  let busy = $state(false);
  let message = $state('');
  onMount(() => {
    const standalone = window.matchMedia('(display-mode: standalone)');
    const update = () => {
      installed =
        standalone.matches ||
        Boolean((navigator as Navigator & { standalone?: boolean }).standalone);
    };
    const available = (event: Event) => {
      event.preventDefault();
      prompt = event as InstallPrompt;
    };
    const complete = () => {
      installed = true;
      prompt = null;
    };
    update();
    window.addEventListener('beforeinstallprompt', available);
    window.addEventListener('appinstalled', complete);
    standalone.addEventListener('change', update);
    return () => {
      window.removeEventListener('beforeinstallprompt', available);
      window.removeEventListener('appinstalled', complete);
      standalone.removeEventListener('change', update);
    };
  });
  async function install() {
    if (!prompt) return;
    const pending = prompt;
    prompt = null;
    busy = true;
    try {
      await pending.prompt();
      const choice = await pending.userChoice;
      message =
        choice.outcome === 'accepted'
          ? 'Запрошена установка.'
          : 'Установка отменена. Панель можно добавить через меню браузера.';
    } catch {
      message = 'Чтобы добавить QSync, открой меню браузера.';
    } finally {
      busy = false;
    }
  }
</script>

<section class="card grid gap-3" aria-labelledby="install-heading">
  <div class="flex items-center gap-3">
    <img src="/icon-192.png" alt="" width="44" height="44" />
    <h2 id="install-heading">Приложение QSync</h2>
  </div>
  {#if installed}
    <p>QSync запущен как установленное приложение.</p>
  {:else}
    <p>Добавь панель на домашний экран или в Dock для быстрого доступа.</p>
    {#if prompt || busy}
      <Button onclick={install} disabled={busy}><Download /> Установить QSync</Button>
    {/if}
    <p>
      iPhone / iPad: Safari → Поделиться → На экран «Домой» → Открыть как веб-приложение. Mac: Safari → Файл → Добавить в Dock.
    </p>
    <p>
      Chrome / Edge: выбери «Установить приложение», если пункт доступен. По адресу HTTP браузер может предложить только ярлык.
    </p>
    <p>
      Для управления и предпросмотра телевизор должен быть доступен по сети.
    </p>
    {#if message}<p role="status">{message}</p>{/if}
  {/if}
</section>
