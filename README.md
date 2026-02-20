# rust-acl-service

`rust-acl-service` — MVP реализация Authorization-as-a-Service согласно `SPECIFICATION_V2.md`.

## Что реализовано

- RBAC-модель в PostgreSQL (`roles`, `permissions`, `role_permissions`, `actor_roles`).
- REST API:
  - `POST /v1/check`;
  - `POST /v1/admin/roles`;
  - `POST /v1/admin/permissions`;
  - `POST /v1/admin/role-permissions`;
  - `POST /v1/admin/actor-roles`.
- Эксплуатационные endpoint'ы: `/health`, `/ready`, `/metrics`.
- Fail-closed логика (`/v1/check` возвращает `allow=false` при внутренних ошибках).
- Интеграционный тест happy-path + deny-path.

## Быстрый старт

1. Запустить PostgreSQL:
   ```bash
   docker compose up -d postgres
   ```
2. Установить переменные окружения:
   ```bash
   export DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/acl
   export HTTP_ADDR=0.0.0.0:8080
   export METRICS_ADDR=0.0.0.0:9000
   ```
3. Запустить сервис:
   ```bash
   cargo run
   ```

Полная документация: [`docs/SERVICE_DOCUMENTATION.md`](docs/SERVICE_DOCUMENTATION.md).
