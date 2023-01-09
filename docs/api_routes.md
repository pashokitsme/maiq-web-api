# Получить снапшот по uid
`GET` `/api/snapshot/<uid>`

> Параметры:
* `uid`: строка, обязательный

> Ответы:
* 200 OK: [`Snapshot`](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md#jsonsnapshot)
* Остальное: [`ApiError`](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md#jsonapierror)
  
# Получить последний снапшот
`GET` `/api/latest/<day>`

> Параметры:
* `day:` 
  * `today` - сегодня
  * `next` - следующий день (завтра/понедельник)
  * `tomorrow` - то же самое, что и next

> Ответы:
* 200 OK: [`Snapshot`](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md#jsonsnapshot)
* Остальное: [`ApiError`](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md#jsonapierror)

# Получить последнее расписание группы
`GET` `/api/latest/<day>/<group>`

> Параметры:
* `day:` 
  * `today` - сегодня
  * `next` - следующий день (завтра/понедельник)
  * `tomorrow` - то же самое, что и next
* `group` - название группы

> Ответы:
* 200 OK: [`TinySnapshot`](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md#jsontinysnapshot)
* Остальное: [`ApiError`](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md#jsonapierror)

# Получить uid последних снапшотов на сегодня и на следующий день
`GET` `/api/poll`

> Ответы:
* 200 OK: [`JSON/Poll`](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md#jsonpoll)
* Остальное: [`ApiError`](https://github.com/pashokitsme/maiq-web-api/blob/master/docs/api_returns.md#jsonapierror)