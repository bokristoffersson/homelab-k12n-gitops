# Corpus Schema

## Command Card Schema

Each command card is a YAML file under `cards/` with the following structure:

### Required Fields

- **id** (string): Unique identifier (e.g., `restart-deployment`)
- **title** (string): Human-readable title (e.g., "Restart a Deployment")
- **intent** (enum): Primary operation intent
  - Values: `diagnose`, `logs`, `restart`, `scale`, `cordon`, `drain`, `uncordon`, `status`, `describe`, `events`, `top`
- **resource** (enum): Kubernetes resource type
  - Values: `pod`, `deployment`, `statefulset`, `daemonset`, `node`, `service`, `ingress`, `pvc`, `namespace`
- **risk_level** (enum): Safety classification
  - Values: `none`, `low`, `medium`, `high`

### Core Content

- **command_template** (string): Template with `{{placeholders}}`
  - Must include `-n {{namespace}}` for namespaced resources
  - Example: `kubectl rollout restart deployment/{{name}} -n {{namespace}}`

- **preconditions** (list[string]): Checks required before execution
  - Example: `["Deployment must exist", "User has edit permissions"]`

### Configuration

- **flags** (list[object]): Optional command flags
  - **name** (string): Flag name
  - **default** (string): Default value
  - **note** (string): Usage guidance

- **defaults** (object): Default values for template variables
  - Example: `{ timeout: "30s", replicas: 1 }`

### Examples

- **examples** (list[object]): Worked examples
  - **goal** (string): What the example accomplishes
  - **render** (object):
    - **command** (string): Rendered command
    - **checks** (list[string]): Post-execution verification steps

### Metadata

- **postprocess** (object): Output handling
  - **summarize_to_json** (bool): Parse output as JSON

- **notes** (list[string]): Implementation notes and warnings

- **references** (list[string]): Documentation URLs

## Organization Configuration Schema

### Namespaces (`org/namespaces.yaml`)

```yaml
default_namespace: staging
namespaces:
  - name: prod
    allowed_verbs: [get, describe, logs, rollout]
  - name: staging
    allowed_verbs: [get, describe, logs, rollout, scale, delete]
```

### Label Conventions (`org/label-conventions.yaml`)

```yaml
selectors:
  app_key: app.kubernetes.io/name
  component_key: app.kubernetes.io/component
  fallbacks:
    - app
    - name
```

### RBAC Allow-list (`org/rbac-allowlist.yaml`)

```yaml
allowed_verbs:
  - get
  - describe
  - logs
  - rollout
  - scale
  - cordon
  - drain

allowed_resources:
  - pod
  - deployment
  - statefulset
  - node
  - service

namespace_scoped: true
```

### Enumerations (`org/enums.yaml`)

```yaml
environments: [dev, staging, prod]
tiers: [frontend, backend, data, platform]
exposures: [public, internal, private]
domains: [api, web, worker, data]
slo: [critical, high, standard, low]
```

## Validation Rules

1. All templates must be namespace-aware (include `-n {{namespace}}`)
2. Risk level `high` requires explicit confirmation
3. Intent and resource must match allowed combinations
4. Examples must demonstrate actual template rendering
5. Commands must start with `kubectl`
