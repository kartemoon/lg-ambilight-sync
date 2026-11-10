# Установка LG Hue Sync Custom из IPK

Пакет содержит приложение webOS, ARMv7-демон и встроенную русскую веб-панель. Требуется LG webOS с root-доступом, Homebrew Channel и включённым root SSH. Совместимость захвата зависит от модели и прошивки ТВ. Для торшера Govee C60B0 нужен отдельно настроенный мост ESP32-S3.

1. Скачайте IPK и `SHA256SUMS` со [страницы последнего релиза](https://github.com/kartemoon/lg-hue-sync-custom/releases/latest). В каталоге загрузки выполните `sha256sum -c SHA256SUMS`.
2. Установите IPK через [webOS Dev Manager](https://github.com/webosbrew/dev-manager-desktop) или командой:

   ```sh
   ares-install -d TV_NAME org.webosbrew.lg-hue-sync_1.1.1_all.ipk
   ```

   Замените `TV_NAME` именем устройства в Ares. Используйте имя скачанного IPK, если его версия отличается от примера.
3. Активируйте службу после установки или обновления. Замените `TV_IP` адресом телевизора и `22` портом root SSH, если он отличается.

   ```sh
   ssh -p 22 root@TV_IP 'sh /media/developer/apps/usr/palm/applications/org.webosbrew.lg-hue-sync/install-daemon.sh'
   ```

4. Откройте приложение на ТВ или `http://TV_IP:8088/` с другого устройства в домашней сети. Настройте Hue или Nanoleaf в панели.

Активация сохраняет файлы сопряжения, аудио и калибровки. При обновлении предыдущий бинарник сохраняется как `/var/home/root/lg-hue-sync/lg-hue-sync.previous`. Если новая служба не запускается, скрипт возвращает прежний бинарник. Если папка приложений на ТВ отличается, укажите фактический путь к `install-daemon.sh`.

## Сборка пакета

После сборки панели (`make dashboard-build`) и ARMv7-демона (`make build` или `make build-docker`) запустите:

```sh
python3 scripts/package_ipk.py --daemon target/armv7-unknown-linux-gnueabi/release/lg-hue-sync
```

Требуется [Rust Ares](https://github.com/webosbrew/ares-cli-rs). Результат появится в `target/`. Вариант команды без `--daemon` собирает только ярлык для старого сценария SSH-деплоя.
