```md
# ArchDoc (V1) — Проектный документ для разработки
**Формат:** PRD + Tech Spec (Python-only, CLI-only)  
**Стек реализации:** Rust (CLI), анализ Python через AST, генерация Markdown (diff-friendly)  
**Дата:** 2026-01-25

---

## 1. Контекст и проблема

### 1.1. Боль
- Документация архитектуры и связей в кодовой базе устаревает практически сразу.
- В новых чатах LLM не имеет контекста проекта и не понимает “рельсы”: где что лежит, какие модули, какие зависимости критичны.
- В MR/PR сложно быстро оценить архитектурный impact: что поменялось в зависимостях, какие точки “пробило” изменения.

### 1.2. Цель
Сделать CLI-инструмент, который по существующему Python-проекту генерирует и поддерживает **человеко- и LLM-читаемую** документацию:
- от верхнего уровня (папки, модули, “рельсы”)  
- до **уровня функций/методов** (что делают и с чем связаны)  
при этом обновление должно быть **детерминированным** и **diff-friendly**.

---

## 2. Видение продукта

**ArchDoc** — CLI на Rust, который:
1) сканирует репозиторий Python-проекта,  
2) строит модель модулей/файлов/символов и связей (imports + best-effort calls),  
3) генерирует/обновляет набор Markdown-файлов так, чтобы `git diff` показывал **смысловые** изменения,  
4) создаёт “Obsidian-style” навигацию по ссылкам: индекс → модуль → файл → символ (function/class/method).

---

## 3. Область охвата (V1)

### 3.1. In-scope (обязательно)
- Только **CLI** (без MCP/GUI в V1).
- Только **Python** (в дальнейшем расширяемость под другие языки).
- Документация:
  - `ARCHITECTURE.md` как входная точка,
  - детальные страницы по модулям и файлам,
  - детализация по символам (functions/classes/methods) с связями.
- Связи:
  - dependency graph по импортам модулей,
  - best-effort call graph на уровне файла/символа,
  - inbound/outbound зависимости (кто зависит / от кого зависит).
- Diff-friendly обновление:
  - маркерные секции,
  - перезапись только генерируемых блоков,
  - стабильные ID и сортировки.

### 3.2. Out-of-scope (V1)
- MCP, IDE-интеграции.
- Полный семантический резолв вызовов (уровень LSP/type inference) — только best-effort.
- Визуальная “сеточка графа” — в roadmap (V2+).
- LLM-суммаризация кода — V1 не должен “придумывать”; описание берём из docstring + эвристика.

---

## 4. Основные термины

### 4.1. Symbol (символ)
Именованная сущность, которой можно адресно дать документацию и связи:
- `function` / `async function` (def/async def),
- `class`,
- `method` (внутри class),
- (опционально) module/package как верхнеуровневые сущности.

**Symbol ≠ вызов.**  
Symbol — это **определение**, call/reference — **использование**.

---

## 5. Пользовательские сценарии

### S1. init
Пользователь выполняет `archdoc init`:
- создаётся `ARCHITECTURE.md` (в корне проекта),
- создаётся `archdoc.toml` (рекомендуемо) и директория `docs/architecture/*` (если нет).

### S2. generate/update
Пользователь выполняет `archdoc generate` (или `archdoc update`):
- анализирует репозиторий,
- создаёт/обновляет Markdown-артефакты,
- в MR/PR дифф отражает только смысловые изменения.

### S3. check (CI)
`archdoc check`:
- завершает процесс с non-zero кодом, если текущие docs не соответствуют тому, что будет сгенерировано.

---

## 6. Продуктовые принципы (не обсуждаются)

1) **Детерминизм:** один и тот же вход → один и тот же выход.  
2) **Diff-friendly:** минимальный шум в `git diff`.  
3) **Ручной контент не затираем:** всё вне маркеров — зона ответственности человека.  
4) **Без “галлюцинаций”:** связи выводим только из анализа (AST + индекс), иначе помечаем как unresolved/external.  
5) **Масштабируемость:** кеширование, инкрементальные обновления, параллельная обработка.

---

## 7. Артефакты вывода

### 7.1. Структура файлов (рекомендуемая)
```

ARCHITECTURE.md
docs/
architecture/
_index.md
rails.md
layout.md
modules/
<module_id>.md
files/
<path_sanitized>.md

````

### 7.2. Обязательные требования к контенту
- `ARCHITECTURE.md` содержит:
  - название, описание (manual),
  - Created/Updated (Updated меняется **только если** изменилась любая генерируемая секция),
  - rails/tooling,
  - layout,
  - индекс модулей,
  - критичные dependency points (fan-in/fan-out/cycles).
- `modules/<module_id>.md` содержит:
  - intent (manual),
  - boundaries (генерируемое),
  - deps inbound/outbound (генерируемое),
  - symbols overview (генерируемое).
- `files/<path>.md` содержит:
  - intent (manual),
  - file imports + deps (генерируемое),
  - индекс symbols в файле,
  - **один блок на каждый symbol** с назначением и связями.

---

## 8. Diff-friendly обновление (ключевое)

### 8.1. Маркерные секции
Любая генерируемая часть окружена маркерами:

- `<!-- ARCHDOC:BEGIN section=<name> -->`
- `<!-- ARCHDOC:END section=<name> -->`

Для символов:
- `<!-- ARCHDOC:BEGIN symbol id=<symbol_id> -->`
- `<!-- ARCHDOC:END symbol id=<symbol_id> -->`

Инструмент **обновляет только содержимое внутри** этих маркеров.

### 8.2. Ручные секции
Рекомендуемый паттерн:
- `<!-- MANUAL:BEGIN -->`
- `<!-- MANUAL:END -->`

Инструмент не трогает текст в этих блоках и вообще не трогает всё, что вне `ARCHDOC` маркеров.

### 8.3. Детерминированные сортировки
- списки модулей/файлов/символов сортируются лексикографически по стабильному ключу,
- таблицы имеют фиксированный набор колонок и формат,
- запрещены “плавающие” элементы (кроме Updated, который обновляется только при изменениях).

### 8.4. Updated-таймстамп без шума
Правило V1:
- пересчитать контент-хеш генерируемых секций,
- **если** он изменился → обновить `Updated`,
- **иначе** не менять дату.

---

## 9. Stable IDs и якоря

### 9.1. Symbol ID
Формат:
- `py::<module_path>::<qualname>`

Примеры:
- `py::app.billing::apply_promo_code`
- `py::app.services.user::UserService.create_user`

Коллизии:
- добавить `#<short_hash>` (например, от сигнатуры/позиции).

### 9.2. File doc имя
`<relative_path>` конвертируется в:
- `files/<path_sanitized>.md`
- где `path_sanitized` = заменить `/` на `__`

Пример:
- `src/app/billing.py` → `docs/architecture/files/src__app__billing.py.md`

### 9.3. Якоря
Внутри file docs якорь для symbol:
- `#<anchor>` где `<anchor>` = безопасная форма от symbol_id
- дополнительно можно вставить `<a id="..."></a>`.

---

## 10. Python анализ (V1)

### 10.1. Что считаем модулем
- Python package: директория с `__init__.py`
- module: `.py` файл, который принадлежит package/root

Поддержка src-layout:
- конфиг `src_roots = ["src", "."]`

### 10.2. Извлекаем из AST (обязательно)
- `import` / `from ... import ...` + алиасы
- определения: `def`, `async def`, `class`, методы в классах
- docstring (первая строка как “краткое назначение”)
- сигнатура: аргументы, defaults, аннотации типов, return annotation (если есть)

### 10.3. Call graph (best-effort, без type inference)
Резолв вызовов:
- `Name()` вызов `foo()`:
  - если `foo` определён в этом файле → связываем на локальный symbol,
  - если `foo` импортирован через `from x import foo` (или алиас) → связываем на `x.foo`,
  - иначе → `external_call::foo`.
- `Attribute()` вызов `mod.foo()`:
  - если `mod` — импортированный модуль/алиас → резолвим к `mod.foo`,
  - иначе → `unresolved_method_call::mod.foo`.

Важно: лучше пометить как unresolved, чем “натянуть” неверную связь.

### 10.4. Inbound связи (кто зависит)
- на уровне модулей/файлов: строим обратный граф импортов
- на уровне symbols: строим обратный граф calls там, где вызовы резолвятся

---

## 11. “Что делает функция” (без LLM)

### 11.1. Источник истины: docstring
- `purpose.short` = первая строка docstring
- `purpose.long` (опционально) = первые N строк docstring

### 11.2. Эвристика (если docstring нет)
- по имени: `get_*`, `create_*`, `update_*`, `delete_*`, `sync_*`, `validate_*`
- по признакам в AST:
  - наличие HTTP клиентов (`requests/httpx/aiohttp`),
  - DB libs (`sqlalchemy/peewee/psycopg/asyncpg`),
  - tasks/queue (`celery`, `kafka`, `pika`),
  - чтение/запись файлов (`open`, `pathlib`),
  - raising exceptions, early returns.
Формат результата: одна строка с меткой `[heuristic]`.

### 11.3. Manual override
- секция “Manual notes” для каждого symbol — зона ручного уточнения.

---

## 12. CLI спецификация

### 12.1. Команды
- `archdoc init`
  - создаёт `ARCHITECTURE.md`, `docs/architecture/*`, `archdoc.toml` (если нет)
- `archdoc generate` / `archdoc update`
  - анализ + запись/обновление файлов
- `archdoc check`
  - проверка: docs совпадают с тем, что будет сгенерировано

### 12.2. Флаги (V1)
- `--root <path>` (default: `.`)
- `--out <path>` (default: `docs/architecture`)
- `--config <path>` (default: `archdoc.toml`)
- `--verbose`
- `--include-tests/--exclude-tests` (можно через конфиг)

---

## 13. Конфигурация (`archdoc.toml`)

Минимальный конфиг V1:
```toml
[project]
root = "."
out_dir = "docs/architecture"
entry_file = "ARCHITECTURE.md"
language = "python"

[scan]
include = ["src", "app", "tests"]
exclude = [".venv", "venv", "__pycache__", ".git", "dist", "build", ".mypy_cache", ".ruff_cache"]
follow_symlinks = false

[python]
src_roots = ["src", "."]
include_tests = true

[output]
single_file = false
per_file_docs = true

[diff]
update_timestamp_on_change_only = true

[thresholds]
critical_fan_in = 20
critical_fan_out = 20
````

---

## 14. Шаблоны Markdown (V1)

### 14.1. `ARCHITECTURE.md` (skeleton)

(Важное: ручные блоки + маркерные генерируемые секции.)

```md
# ARCHITECTURE — <PROJECT_NAME>

<!-- MANUAL:BEGIN -->
## Project summary
**Name:** <PROJECT_NAME>  
**Description:** <FILL_MANUALLY: what this project does in 3–7 lines>

## Key decisions (manual)
- <FILL_MANUALLY>

## Non-goals (manual)
- <FILL_MANUALLY>
<!-- MANUAL:END -->

---

## Document metadata
- **Created:** <AUTO_ON_INIT: YYYY-MM-DD>
- **Updated:** <AUTO_ON_CHANGE: YYYY-MM-DD>
- **Generated by:** archdoc (cli) v0.1

---

## Rails / Tooling
<!-- ARCHDOC:BEGIN section=rails -->
> Generated. Do not edit inside this block.
<AUTO: rails summary + links to config files>
<!-- ARCHDOC:END section=rails -->

---

## Repository layout (top-level)
<!-- ARCHDOC:BEGIN section=layout -->
> Generated. Do not edit inside this block.
<AUTO: table of top-level folders + heuristic purpose + link to layout.md>
<!-- ARCHDOC:END section=layout -->

---

## Modules index
<!-- ARCHDOC:BEGIN section=modules_index -->
> Generated. Do not edit inside this block.
<AUTO: table modules + deps counts + links to module docs>
<!-- ARCHDOC:END section=modules_index -->

---

## Critical dependency points
<!-- ARCHDOC:BEGIN section=critical_points -->
> Generated. Do not edit inside this block.
<AUTO: top fan-in/out symbols + cycles>
<!-- ARCHDOC:END section=critical_points -->

---

<!-- MANUAL:BEGIN -->
## Change notes (manual)
- <FILL_MANUALLY>
<!-- MANUAL:END -->
```

### 14.2. `docs/architecture/layout.md`

```md
# Repository layout

<!-- MANUAL:BEGIN -->
## Manual overrides
- `src/app/` — <FILL_MANUALLY>
<!-- MANUAL:END -->

---

## Detected structure
<!-- ARCHDOC:BEGIN section=layout_detected -->
> Generated. Do not edit inside this block.
<AUTO: table of paths>
<!-- ARCHDOC:END section=layout_detected -->
```

### 14.3. `docs/architecture/modules/<module_id>.md`

```md
# Module: <module_id>

- **Path:** <AUTO>
- **Type:** python package/module
- **Doc:** <AUTO: module docstring summary if any>

<!-- MANUAL:BEGIN -->
## Module intent (manual)
<FILL_MANUALLY: boundaries, responsibility, invariants>
<!-- MANUAL:END -->

---

## Dependencies
<!-- ARCHDOC:BEGIN section=module_deps -->
> Generated. Do not edit inside this block.
<AUTO: outbound/inbound modules + counts>
<!-- ARCHDOC:END section=module_deps -->

---

## Symbols overview
<!-- ARCHDOC:BEGIN section=symbols_overview -->
> Generated. Do not edit inside this block.
<AUTO: table of symbols + links into file docs>
<!-- ARCHDOC:END section=symbols_overview -->
```

### 14.4. `docs/architecture/files/<path_sanitized>.md`

```md
# File: <relative_path>

- **Module:** <AUTO: module_id>
- **Defined symbols:** <AUTO>
- **Imports:** <AUTO>

<!-- MANUAL:BEGIN -->
## File intent (manual)
<FILL_MANUALLY>
<!-- MANUAL:END -->

---

## Imports & file-level dependencies
<!-- ARCHDOC:BEGIN section=file_imports -->
> Generated. Do not edit inside this block.
<AUTO: imports list + outbound modules + inbound files>
<!-- ARCHDOC:END section=file_imports -->

---

## Symbols index
<!-- ARCHDOC:BEGIN section=symbols_index -->
> Generated. Do not edit inside this block.
<AUTO: list of links to symbol anchors>
<!-- ARCHDOC:END section=symbols_index -->

---

## Symbol details

<!-- ARCHDOC:BEGIN symbol id=py::<module>::<qualname> -->
<a id="<anchor>"></a>

### `py::<module>::<qualname>`
- **Kind:** function | class | method
- **Signature:** `<AUTO>`
- **Docstring:** `<AUTO: first line | No docstring>`
- **Defined at:** `<AUTO: line>` (optional)

#### What it does
<!-- ARCHDOC:BEGIN section=purpose -->
<AUTO: docstring-first else heuristic with [heuristic]>
<!-- ARCHDOC:END section=purpose -->

#### Relations
<!-- ARCHDOC:BEGIN section=relations -->
**Outbound calls (best-effort):**
- <AUTO: resolved symbol ids>
- external_call::<name>
- unresolved_method_call::<expr>

**Inbound (used by) (best-effort):**
- <AUTO: callers>
<!-- ARCHDOC:END section=relations -->

#### Integrations (heuristic)
<!-- ARCHDOC:BEGIN section=integrations -->
- HTTP: yes/no
- DB: yes/no
- Queue/Tasks: yes/no
<!-- ARCHDOC:END section=integrations -->

#### Risk / impact
<!-- ARCHDOC:BEGIN section=impact -->
- fan-in: <AUTO:int>
- fan-out: <AUTO:int>
- cycle participant: <AUTO: yes/no>
- critical: <AUTO: yes/no + reason>
<!-- ARCHDOC:END section=impact -->

<!-- MANUAL:BEGIN -->
#### Manual notes
<FILL_MANUALLY>
<!-- MANUAL:END -->

<!-- ARCHDOC:END symbol id=py::<module>::<qualname> -->
```

---

## 15. Техническая архитектура реализации (Rust)

### 15.1. Модули приложения (рекомендуемое разбиение crates/modules)

* `cli` — парсинг аргументов, команды init/generate/check
* `scanner` — обход файлов, ignore, include/exclude
* `python_analyzer` — AST парсер/индексатор (Python)
* `model` — IR структуры данных (ProjectModel)
* `renderer` — генерация Markdown (шаблоны)
* `writer` — diff-aware writer: обновление по маркерам
* `cache` — кеш по хешам файлов (опционально в V1, но желательно)

### 15.2. IR (Intermediate Representation) — схема данных

Минимальные сущности:

**ProjectModel**

* modules: Map<module_id, Module>
* files: Map<file_id, FileDoc>
* symbols: Map<symbol_id, Symbol>
* edges:

  * module_import_edges: Vec<Edge> (module → module)
  * file_import_edges: Vec<Edge> (file → module/file)
  * symbol_call_edges: Vec<Edge> (symbol → symbol/external/unresolved)

**Module**

* id, path, files[], doc_summary
* outbound_modules[], inbound_modules[]
* symbols[]

**FileDoc**

* id, path, module_id
* imports[] (normalized)
* outbound_modules[], inbound_files[]
* symbols[]

**Symbol**

* id, kind, module_id, file_id, qualname
* signature (string), annotations (optional structured)
* docstring_first_line
* purpose (docstring/heuristic)
* outbound_calls[], inbound_calls[]
* integrations flags
* metrics: fan_in, fan_out, is_critical, cycle_participant

**Edge**

* from_id, to_id, edge_type, meta (optional)

---

## 16. Алгоритмы (ключевые)

### 16.1. Scanner

* применить exclude/include и игноры
* собрать список `.py` файлов
* определить src_root и module paths

### 16.2. Python Analyzer

Шаги:

1. Пройти по каждому `.py` файлу
2. Распарсить AST
3. Извлечь:

   * imports + алиасы
   * defs/classes/methods + сигнатуры + docstrings
   * calls (best-effort)
4. Построить Symbol Index: `name → symbol_id` в рамках файла и модуля
5. Резолвить calls через:

   * локальные defs
   * from-import алиасы
   * import module алиасы
6. Построить edges, затем обратные edges (inbound)

### 16.3. Writer (diff-aware)

* загрузить существующий md (если есть)
* найти маркеры секций
* заменить содержимое секции детерминированным рендером
* сохранить всё вне маркеров неизменным
* если файл отсутствует → создать по шаблону
* пересчитать общий “генерируемый хеш”:

  * если изменился → обновить `Updated`, иначе оставить

---

## 17. Критичные точки (impact analysis)

Метрики:

* **fan-in(symbol)** = число inbound вызовов (resolved)
* **fan-out(symbol)** = число outbound вызовов (resolved + unresolved по отдельному счётчику)
* **critical**:

  * `fan-in >= thresholds.critical_fan_in` OR
  * `fan-out >= thresholds.critical_fan_out` OR
  * участие в цикле модулей

Выводить top-N списки в `ARCHITECTURE.md`.

---

## 18. Нефункциональные требования

* Время генерации: приемлемо на средних репо (ориентир — минуты, с перспективой кеширования).
* Память: не грузить весь исходный текст в память надолго; хранить только необходимое.
* Безопасность: по умолчанию не включать секреты/бинарники; уважать exclude.
* Надёжность: если AST не парсится (битый файл) — лог + продолжить анализ остальных, пометив файл как failed.

---

## 19. Acceptance Criteria (V1)

1. `archdoc init` создаёт:

   * `ARCHITECTURE.md` с manual блоками и маркерами секций
   * `docs/architecture/*` с базовыми файлами (или создаёт при generate)

2. Повторный `archdoc generate` на неизменном репо даёт:

   * нулевой diff (включая `Updated`, который не меняется без контентных изменений)

3. Изменение одной функции/файла приводит:

   * к локальному diff только соответствующего symbol блока и агрегатов (indexes/critical points)

4. `archdoc check` корректно детектит рассинхронизацию и возвращает non-zero.

---

## 20. План релизов (Roadmap)

### V1 (текущий документ)

* Python-only CLI
* modules/files/symbols docs
* import graph + best-effort call graph
* diff-friendly writer
* init/generate/check

### V2 (следующий шаг)

* Экспорт графа в JSON/Mermaid
* Простая локальная HTML/MD визуализация “как в Obsidian” (сетка зависимостей)
* Улучшение резолва calls (больше случаев через алиасы/простые типы)

### V3+

* Подключение других языков (через tree-sitter провайдеры)
* Опционально LSP режим для точного call graph
* MCP/IDE интеграции

---

## 21. Backlog (V1 — минимально достаточный)

### Эпик A — CLI и конфиг

* A1: `init` создаёт skeleton + config
* A2: `generate/update` парсит конфиг и пишет docs
* A3: `check` сравнивает с виртуально сгенерированным выводом

### Эпик B — Python анализ

* B1: scanner и определение module paths
* B2: AST import extraction + алиасы
* B3: defs/classes/methods extraction + signatures/docstrings
* B4: call extraction + best-effort resolution
* B5: inbound/outbound построение графов

### Эпик C — Markdown генерация и writer

* C1: renderer шаблонов
* C2: marker-based replace секций
* C3: stable sorting и формат таблиц
* C4: update timestamp on change only

### Эпик D — Critical points

* D1: fan-in/fan-out метрики
* D2: top lists в ARCHITECTURE.md
* D3: module cycles detection (простая графовая проверка)

---

## 22. Примечания по качеству (сразу закладываем тестируемость)

* Golden-tests: на маленьком fixture repo хранить ожидаемые md и проверять детерминизм.
* Unit-tests на writer: заменить секцию без изменения остального файла.
* Unit-tests на import/call resolution: алиасы `import x as y`, `from x import a as b`.

---

## 23. Итог

V1 фиксирует базовый продукт: **полная архитектурная документация до уровня функций** с зависимостями и impact, обновляемая безопасно и читаемо через `git diff`. Инструмент закрывает задачу: дать LLM и человеку стабильную “карту проекта” и контролировать критичные точки при изменениях.

---

```
```
