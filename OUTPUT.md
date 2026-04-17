# Output shapes

Human-readable mirror of the per-command JSON shapes the CLI emits on stdout. The live source of truth is `OUTPUT_SHAPES` in `src/schema.rs`. The CLI exposes the same data at runtime via `posthog schema` and `posthog <cmd> --help --json`.

## config

### `posthog config set`

- Type: object
- Description: The saved config (apiKey is stored in plaintext on disk).

| Field | Type |
|---|---|
| apiKey | string |
| projectId | string |
| host | string |

### `posthog config show`

- Type: object
- Description: The current effective config. apiKey is masked for display.

| Field | Type |
|---|---|
| apiKey | string |
| projectId | string |
| host | string |

## login

### `posthog login`

- Type: object
- Description: The saved config after interactive login. apiKey is masked.

| Field | Type |
|---|---|
| apiKey | string |
| projectId | string |
| host | string |

## flags

### `posthog flags list`

- Type: array
- Description: Array of feature flags.

| Field | Type |
|---|---|
| id | number |
| key | string |
| name | string |
| active | boolean |
| rollout_percentage | number \| null |

### `posthog flags get`

- Type: object
- Description: A single feature flag.

| Field | Type |
|---|---|
| id | number |
| key | string |
| name | string |
| active | boolean |
| rollout_percentage | number \| null |

### `posthog flags create`

- Type: object
- Description: The newly created feature flag.

| Field | Type |
|---|---|
| id | number |
| key | string |
| name | string |
| active | boolean |

### `posthog flags update`

- Type: object
- Description: The updated feature flag.

| Field | Type |
|---|---|
| id | number |
| key | string |
| name | string |
| active | boolean |

### `posthog flags enable`

- Type: object
- Description: The updated feature flag with active=true.

| Field | Type |
|---|---|
| id | number |
| key | string |
| active | boolean |

### `posthog flags disable`

- Type: object
- Description: The updated feature flag with active=false.

| Field | Type |
|---|---|
| id | number |
| key | string |
| active | boolean |

### `posthog flags delete`

- Type: object
- Description: Confirmation of deletion.

| Field | Type |
|---|---|
| deleted | boolean |
| key | string |
| id | number |

## experiments

### `posthog experiments list`

- Type: array
- Description: Array of experiments.

| Field | Type |
|---|---|
| id | number |
| name | string |
| start_date | string \| null |
| end_date | string \| null |

### `posthog experiments get`

- Type: object
- Description: A single experiment.

| Field | Type |
|---|---|
| id | number |
| name | string |
| start_date | string \| null |
| end_date | string \| null |

### `posthog experiments results`

- Type: object
- Description: Raw experiment results payload from the PostHog API.

### `posthog experiments launch`

- Type: object
- Description: The experiment with start_date set to now.

| Field | Type |
|---|---|
| id | number |
| start_date | string |

### `posthog experiments pause`

- Type: object
- Description: The experiment with end_date set to now.

| Field | Type |
|---|---|
| id | number |
| end_date | string |

### `posthog experiments end`

- Type: object
- Description: The experiment with end_date set to now.

| Field | Type |
|---|---|
| id | number |
| end_date | string |

## insights

### `posthog insights list`

- Type: array
- Description: Array of insights.

| Field | Type |
|---|---|
| id | number |
| name | string |
| short_id | string |

### `posthog insights get`

- Type: object
- Description: A single insight.

| Field | Type |
|---|---|
| id | number |
| name | string |
| short_id | string |

## dashboards

### `posthog dashboards list`

- Type: array
- Description: Array of dashboards.

| Field | Type |
|---|---|
| id | number |
| name | string |

### `posthog dashboards get`

- Type: object
- Description: A single dashboard.

| Field | Type |
|---|---|
| id | number |
| name | string |

## query

### `posthog query`

- Type: object
- Description: Raw HogQL query result from PostHog. Typically `{ results: any[][], columns: string[], types: string[] }`.

## schema

### `posthog schema`

- Type: object
- Description: The CLI schema itself (this command's output).

---

When you change a command's output, update both `OUTPUT_SHAPES` in `src/schema.rs` AND this doc.
