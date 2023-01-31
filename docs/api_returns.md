> **Все значения даты/время возвращаются в формате `UTC` без указания часового пояса, но, тем не менее, они все `UTC+3`**

## Snapshot ([src](https://github.com/pashokitsme/maiq-parser/blob/master/maiq-shared/src/lib.rs))

```json5
{
  "date": "2023-01-19T00:00:00Z", // время, на которое предназначается снапшот
  "parsed_date": "2023-01-18T12:43:31.459277422Z", // время, когда снапшот добавлен в базу
  "uid": "taq0qyb1y4", // уникальный ID, результат натравливания sha256 на все пары всех групп
  "groups": [
    {
      "uid": "oxveklqx7k", // uid для группы
      "name": "Ир1-21",
      "lessons": [
        {
          "num": 1, // номер пары
          "name": "По расписанию" // может быть автоматически заменено, если указано в /default/<day>.json
        },
        {
          "num": 2,
          "name": "Теория вероятностей и математическая статистика",
          "teacher": "Петрова Н.Г.",
          "classroom": "304У"
        },
        ...
      ]
    },
    ...
  ]
}
```

## TinySnapshot ([src](https://github.com/pashokitsme/maiq-parser/blob/master/maiq-shared/src/lib.rs))
Почти то же самое, что и JSON/Snapshot, но хранит в себе только одну группу
```json5
{
  "date": "2023-01-19T00:00:00Z", // время, на которое предназначается снапшот
  "parsed_date": "2023-01-18T12:43:31.459277422Z", // время, когда снапшот добавлен в базу
  "uid": "taq0qyb1y4", // уникальный ID, результат натравливания sha256 на все пары всех групп
  "group":
  {
    "uid": "py65x5aa11",
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
}
```

## DefaultDay ([src](https://github.com/pashokitsme/maiq-parser/blob/master/maiq-shared/src/default.rs))
```json5
{
  "name": "Ир3-21",
  "lessons": [
    {
      "num": 1,
      "name": "Компьютерные сети",
      "is_even": true, // пара на чётной(числителе) неделе
      "teacher": "Васильева И.С.",
      "classroom": "207М"
    },
    {
      "num": 1,
      "name": "Элементы высшей математики",
      "is_even": false, // пара на нечётной(знаменателе) неделе
      "teacher": "Баранова О.Б.",
      "classroom": "102О"
    },
    {
      "num": 2,
      "name": "Основы алгоритмизации и программирования",
      "teacher": "Федорова Л.В.",
      "classroom": "204М"
    },
    ...
  ]
}
```

## Poll ([src](https://github.com/pashokitsme/maiq-web-api/blob/master/src/cache.rs)) (deprecated!, спрашивай в телеге)
```json5
{
  "today": {
    "uid": "taq0qyb1y4", // uid снапшота
    "groups": {
      "Кс5-20": "6xnhx4u4c4", // uid группы
      "Ир1-19": "2sop49wlkc",
      ...
    }
  },
  "next": {
    "uid": "ql784g18li",
    "groups": {
      "Ип1-20": "73xpbqp0di",
      "Са3-21": "grra9ogvy4",
      ...
    }
  },
  "last_update": "2023-01-19T19:25:28.728701501Z", // последнее обновление
  "next_update": "2023-01-19T19:28:31.416519196Z" // следующее обновление
}
```

## ApiError ([src](https://github.com/pashokitsme/maiq-web-api/blob/master/src/api/error.rs))
```json5
{
  "cause": "route_not_matched", // ошибка
  "desc": "Failed to match (GET) /api/not_existing_path. Try something else?" // описание ошибки
}
```

> Возможные варианты ошибок:
* `401` `unauthorized`: тебе сюда нельзя
* `404` `route_not_matched`: путь не найден или неправильный параметр
* `404` `snapshot_not_found`: снапшот не найден
* `404` `default_not_found`: нет стандартного расписания
* `500` `db_err`: какая-то х-ня с базой
* `500` `internal_parser_err`: какая-то х-ня с парсером
* `500` `unknown`: 🤔