**Все значения даты/время возвращаются в формате `UTC` без указания часового пояса, но, тем не менее, они все `UTC+3`**

## JSON/Snapshot ([src](https://github.com/pashokitsme/maiq-parser/blob/master/maiq-shared/src/lib.rs#L28-L35))

```json5
"uid": "Ne6THIVKpTdFL0Nx1rSZeyIQ0TcAfR1B", // уникальный ID, результат натравливания sha256 на все пары всех групп
"date": "2022-12-28T00:00:00Z", // дата, для которой предназначается снапшот
"parsed_date": "2022-12-28T03:11:52.535Z", // дата, когда снапшот был спарсен
"is_week_even": true, // чётная или нечётная неделя (числитель или знаменатель)
"groups": [
  {
    "name": "Ир1-19",
    "lessons": [
      {
        "num": 1,
        "name": "Стандартизация, сертификация и техническое документоведение",
        "teacher": "Юшина И.В.", // не сериализуется, если null (зачастую при name = Нет или По расписанию)
        "classroom": "208М", // не сериализуется, если null (зачастую при name = Нет или По расписанию)
      },
      ...
    ]
  }
  ...
] 
```

## JSON/TinySnapshot ([src](https://github.com/pashokitsme/maiq-web-api/blob/master/src/api/mod.rs#L75-L81))
Почти то же самое, что и JSON/Snapshot, но хранит в себе только одну группу
```json5
"uid": "Ne6THIVKpTdFL0Nx1rSZeyIQ0TcAfR1B", // уникальный ID, результат натравливания sha256 на все пары всех групп
"date": "2022-12-28T00:00:00Z", // дата, для которой предназначается снапшот
"parsed_date": "2022-12-28T03:11:52.535Z", // дата, когда снапшот был спарсен
"is_week_even": true, // чётная или нечётная неделя (числитель или знаменатель)
"group":
{
  "name": "Ир1-19",
  "lessons": [
    {
      "num": 1, // номер пары
      "name": "Стандартизация, сертификация и техническое документоведение",
      "teacher": "Юшина И.В.", // не сериализуется, если null (зачастую при name = Нет или По расписанию)
      "classroom": "208М", // не сериализуется, если null (зачастую при name = Нет или По расписанию)
    },
    ...
  ]
}
```

## JSON/Poll ([src](https://github.com/pashokitsme/maiq-web-api/blob/master/src/cache.rs#L14-L20))
```json5
"latest_today_uid": "Ne6THIVKpTdFL0Nx1rSZeyIQ0TcAfR1B", // может быть null
"latest_next_uid": null, // может быть null
"last_update": "2023-01-09T18:25:22.134321500Z", // время последнего обновления
"next_update": "2023-01-09T18:28:22.841154Z" // время, в которое будет следующее обновление
```

## JSON/ApiError ([src](https://github.com/pashokitsme/maiq-web-api/blob/master/src/api/error.rs#L11-L38))
```json5
  "cause": "route_not_matched", // ошибка
  "desc": "Failed to match (GET) /api/not_existing_path. Try something else?" // описание ошибки
```

Возможные варианты ошибок:
* `401` `unauthorized`: тебе сюда нельзя
* `404` `route_not_matched`: путь не найден или неправильный параметр
* `404` `snapshot_not_found`: снапшот не найден
* `500` `db_err`: какая-то х-ня с базой
* `500` `internal_parser_err`: какая-то х-ня с парсером
* `500` `unknown`: 🤔