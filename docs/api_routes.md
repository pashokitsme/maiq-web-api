# Получить снапшот по uid
`GET` `/api/snapshot/<uid>`

Параметры:
* `uid`: строка, обязательный

Ответы:
* 200 OK: `JSON/Snapshot`
* Остальное: `JSON/ApiError`
  
# Получить последний снапшот
`GET` `/api/latest/<day>`

Параметры:
* `day:` 
* * `today` - сегодня
* * `next` - следующий день (завтра/понедельник)
* * `tomorrow` - то же самое, что и next

Ответы:
* 200 OK: `JSON/Snapshot`
* Остальное: `JSON/ApiError`

# Получить последнее расписание группы
`GET` `/api/latest/<day>/<group>`

Параметры:
* `day:` 
* * `today` - сегодня
* * `next` - следующий день (завтра/понедельник)
* * `tomorrow` - то же самое, что и next
* `group` - название группы

Ответы:
* 200 OK: `JSON/TinySnapshot`
* Остальное: `JSON/ApiError`

# Получить uid последних снапшотов на сегодня и на следующий день
`GET` `/api/poll`

Ответы:
* 200 OK: `JSON/Poll`
* Остальное: `JSON/ApiError`