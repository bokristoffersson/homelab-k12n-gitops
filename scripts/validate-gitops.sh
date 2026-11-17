#!/usr/bin/env bash
# Render every path Flux reconciles and validate the output against the
# Kubernetes and CRD schemas. Used by `make lint`, the prek hook and CI.
set -euo pipefail

KUBERNETES_VERSION="1.36.2"
CRD_SCHEMAS='https://raw.githubusercontent.com/datreeio/CRDs-catalog/main/{{.Group}}/{{.ResourceKind}}_{{.ResourceAPIVersion}}.json'
CLUSTER_DIR="gitops/clusters/homelab"

cd "$(git rev-parse --show-toplevel)"

for tool in kubectl kubeconform; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "error: $tool is not installed (see 'make setup-tools')" >&2
    exit 1
  fi
done

validate() {
  kubeconform -strict -summary -skip CustomResourceDefinition \
    -kubernetes-version "$KUBERNETES_VERSION" \
    -schema-location default \
    -schema-location "$CRD_SCHEMAS" \
    "$@"
}

paths=$(sed -n 's/^ *path: *"\{0,1\}\.\/\([^" #]*\).*$/\1/p' "$CLUSTER_DIR"/*.yaml | sort -u)
if [ -z "$paths" ]; then
  echo "error: found no Flux Kustomization paths in $CLUSTER_DIR" >&2
  exit 1
fi

failed=0

echo "==> $CLUSTER_DIR"
validate "$CLUSTER_DIR" || failed=1

for path in $paths; do
  echo "==> $path"
  if ! rendered=$(kubectl kustomize "$path"); then
    echo "error: kustomize build failed for $path" >&2
    failed=1
    continue
  fi
  validate - <<<"$rendered" || failed=1
done

exit "$failed"
