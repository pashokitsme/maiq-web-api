# Получить снапшот по uid
`GET` `/api/snapshot/<uid>`

> Параметры:
* `uid`

> Ответы:
* 200 OK: [`Snapshot`](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md#snapshot)
* Остальное: [`ApiError`](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md#apierror)
  
# Получить последний снапшот
`GET` `/api/latest/<day>`

> Параметры:
* `day`: 
  * `today`: сегодня
  * `next`: следующий день (завтра/понедельник)
  * `tomorrow`: то же самое, что и next

> Ответы:
* 200 OK: [`Snapshot`](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md#snapshot)
* Остальное: [`ApiError`](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md#apierror)

# Получить последнее расписание группы
`GET` `/api/latest/<day>/<group>`

> Параметры:
* `day`: 
  * `today`: сегодня
  * `next`: следующий день (завтра/понедельник)
  * `tomorrow`: то же самое, что и next
* `group`: название группы

> Ответы:
* 200 OK: [`TinySnapshot`](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md#tinysnapshot)
* Остальное: [`ApiError`](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md#apierror)

# Получить uid последних снапшотов на сегодня и на следующий день
`GET` `/api/poll`

> Ответы:
* 200 OK: [`Poll`](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md#poll)
* Остальное: [`ApiError`](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md#apierror)

# Получить стандартное расписание
`GET` `/api/default/<weekday>/<group>`

> Параметры:
* `weekday`: день недели (`mon`, `tue`, `wed`, `thu`, `fri`, `sat` соответственно)
* `group`: название группы

> Ответы:
  * 200 OK: [`DefaultDay`](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md#defaultday)
  * Остальное: [`ApiError`](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md#apierror)