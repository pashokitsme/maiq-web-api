# Бекенд проекта 

Публичный API для получения расписания 4 корпуса ЧЭМК \
Стек: [**Rocket**](https://rocket.rs/) + **MongoDB** + [**maiq-parser**](https://github.com/pashokitsme/maiq-parser)


## Докуменация

> Хост API: https://maiq.pashok.me/ \
[/docs](https://github.com/pashokitsme/maiq-web-api/tree/master/docs)
>> [/api_routes.md](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_routes.md) - пути API, методы и их параметры \
>> [/api_returns.md](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md) - всевозможные варианты возвращаемых данных <br>

## Сборка
Требования: **perl**, **rustc ^1.66.0** (ниже хз), **cargo** + **stable-msvc** (windows) или **stable** (linux) **toolchain**

> Перед сборкой/запуском, создать **.env** файл конфигурации \
> Обязательные параметры проверяются при запуске

```bash
> cargo build --release
> cd target/release/
> ./maiq-web.exe
```