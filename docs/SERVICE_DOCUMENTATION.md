# rust-acl-service: подробная документация MVP

## 1. Назначение

Сервис реализует **Authorization (AuthZ)** как отдельный компонент и принимает решения доступа на основе RBAC-модели.

### Вход в decision engine
- `actor_id`
- `action`
- `resource`

### Выход
- `allow: bool`

## 2. Архитектура

### 2.1. Слои
1. **HTTP слой (`axum`)**
   - маршруты health/ready/check/admin;
   - сериализация/десериализация JSON.
2. **Application layer**
   - orchestration запроса и метрик;
   - fail-closed поведение.
3. **Data layer (`sqlx` + PostgreSQL)**
   - SQL-операции RBAC;
   - миграции при запуске сервиса.

### 2.2. Fail-closed
При ошибке БД в `POST /v1/check` возвращается `{"allow": false}` вместо fail-open, что соответствует требованиям MVP.

## 3. Схема данных

### `roles`
- `id` PK
- `name` UNIQUE

### `permissions`
- `id` PK
- `action`
- `resource`
- UNIQUE(`action`, `resource`)

### `role_permissions`
- `role_id` FK -> `roles`
- `permission_id` FK -> `permissions`
- PK(`role_id`, `permission_id`)

### `actor_roles`
- `actor_id`
- `role_id` FK -> `roles`
- PK(`actor_id`, `role_id`)

### Индексы
- `idx_actor_roles_actor_id`
- `idx_role_permissions_role_id`
- `idx_permissions_action_resource`

## 4. API контракт

## 4.1. Health/readiness
### `GET /health`
Проверка liveliness процесса.

### `GET /ready`
Проверка доступности БД (`SELECT 1`).

## 4.2. Check API
### `POST /v1/check`
Request:
```json
{
  "actor_id": "user-1",
  "action": "read",
  "resource": "document:123"
}
```

Response:
```json
{
  "allow": true
}
```

## 4.3. Admin API
### `POST /v1/admin/roles`
```json
{"name":"reader"}
```

### `POST /v1/admin/permissions`
```json
{"action":"read","resource":"document:123"}
```

### `POST /v1/admin/role-permissions`
```json
{"role_name":"reader","action":"read","resource":"document:123"}
```

### `POST /v1/admin/actor-roles`
```json
{"actor_id":"user-1","role_name":"reader"}
```

## 5. Метрики и логирование

### Метрики
- `request_count{endpoint="/v1/check"}`
- `request_latency_seconds{endpoint="/v1/check"}`

### Endpoint
- `/metrics` (Prometheus exposition)

### Логи
- JSON structured logs через `tracing-subscriber`.

## 6. Локальный запуск

## 6.1. Через Docker Compose
```bash
docker compose up --build
```

## 6.2. Локально через Cargo
```bash
docker compose up -d postgres
export DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/acl
cargo run
```

## 7. Примеры curl

```bash
curl -X POST http://127.0.0.1:8080/v1/admin/roles \
  -H 'Content-Type: application/json' \
  -d '{"name":"reader"}'

curl -X POST http://127.0.0.1:8080/v1/admin/permissions \
  -H 'Content-Type: application/json' \
  -d '{"action":"read","resource":"document:123"}'

curl -X POST http://127.0.0.1:8080/v1/admin/role-permissions \
  -H 'Content-Type: application/json' \
  -d '{"role_name":"reader","action":"read","resource":"document:123"}'

curl -X POST http://127.0.0.1:8080/v1/admin/actor-roles \
  -H 'Content-Type: application/json' \
  -d '{"actor_id":"user-1","role_name":"reader"}'

curl -X POST http://127.0.0.1:8080/v1/check \
  -H 'Content-Type: application/json' \
  -d '{"actor_id":"user-1","action":"read","resource":"document:123"}'
```

## 8. Интеграционные тесты

Файл: `tests/integration_acl.rs`

Покрытие:
1. health-check;
2. создание role/permission;
3. связка role-permission;
4. назначение роли actor;
5. проверка allow-path;
6. проверка deny-path.

Тест запускает PostgreSQL в контейнере (`testcontainers`) и поднимает `axum` сервер на ephemeral-порту.
