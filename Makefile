# Homelab GitOps Makefile
# Convenience commands for local development and cluster management

.PHONY: help
help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

##@ Local Development

.PHONY: local-up
local-up: ## Create local k3d cluster (no Flux)
	./scripts/setup-local-cluster.sh

.PHONY: local-down
local-down: ## Delete local k3d cluster
	k3d cluster delete homelab-local

.PHONY: local-restart
local-restart: local-down local-up ## Restart local cluster

.PHONY: local-secrets
local-secrets: ## Create local development secrets
	./scripts/create-local-secrets.sh

##@ Direct Development (kubectl apply)

.PHONY: dev-apply-infra
dev-apply-infra: ## Apply infrastructure directly (no Flux)
	kubectl apply -k gitops/infrastructure/controllers-local

.PHONY: dev-apply-apps
dev-apply-apps: ## Apply all apps directly (no Flux)
	kubectl apply -k gitops/apps/local/redpanda-v2
	kubectl apply -k gitops/apps/local/monitoring

.PHONY: dev-apply-redpanda
dev-apply-redpanda: ## Apply just Redpanda directly
	kubectl apply -k gitops/apps/local/redpanda-v2

.PHONY: dev-apply-monitoring
dev-apply-monitoring: ## Apply just monitoring directly
	kubectl apply -k gitops/apps/local/monitoring

.PHONY: dev-delete-redpanda
dev-delete-redpanda: ## Delete Redpanda resources
	kubectl delete -k gitops/apps/local/redpanda-v2

.PHONY: dev-watch
dev-watch: ## Watch all pods in all namespaces
	kubectl get pods -A --watch

##@ Production Flux Commands (not for local dev)

.PHONY: flux-check
flux-check: ## Check Flux prerequisites and status (production)
	@echo "⚠️  This is for production cluster only!"
	flux check

.PHONY: flux-reconcile
flux-reconcile: ## Reconcile Flux kustomizations (production)
	@echo "⚠️  This is for production cluster only!"
	@read -p "Are you sure you're on the production cluster? [y/N] " -n 1 -r; \
	echo; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		flux reconcile source git flux-system; \
		flux reconcile kustomization flux-system; \
	fi

.PHONY: flux-logs
flux-logs: ## Watch Flux logs (production)
	flux logs --all-namespaces --follow

.PHONY: flux-get
flux-get: ## Get all Flux resources (production)
	flux get all --all-namespaces

##@ Kubernetes Commands

.PHONY: k-status
k-status: ## Show cluster status
	@echo "=== Nodes ==="
	kubectl get nodes
	@echo "\n=== Namespaces ==="
	kubectl get namespaces
	@echo "\n=== Flux Kustomizations ==="
	flux get kustomizations

.PHONY: k-pods
k-pods: ## List all pods
	kubectl get pods --all-namespaces

.PHONY: k-events
k-events: ## Show recent events
	kubectl get events --all-namespaces --sort-by='.lastTimestamp' | tail -20

##@ Port Forwarding

.PHONY: port-redpanda
port-redpanda: ## Port-forward Redpanda Console (8080)
	@echo "Opening Redpanda Console at http://localhost:8080"
	kubectl port-forward -n redpanda-v2 svc/redpanda-v2-console 8080:8080

.PHONY: port-mosquitto
port-mosquitto: ## Port-forward Mosquitto MQTT (1883)
	@echo "Port-forwarding Mosquitto MQTT at localhost:1883"
	kubectl port-forward -n mosquitto svc/mosquitto 1883:1883

.PHONY: port-traefik
port-traefik: ## Port-forward Traefik dashboard (9000)
	@echo "Opening Traefik dashboard at http://localhost:9000/dashboard/"
	kubectl port-forward -n traefik svc/traefik 9000:9000

.PHONY: port-prometheus
port-prometheus: ## Port-forward Prometheus (9090)
	@echo "Opening Prometheus at http://localhost:9090"
	kubectl port-forward -n monitoring svc/prometheus-kube-prometheus-prometheus 9090:9090

.PHONY: port-grafana
port-grafana: ## Port-forward Grafana (3000)
	@echo "Opening Grafana at http://localhost:3000"
	kubectl port-forward -n monitoring svc/grafana 3000:80

##@ MQTT Generator

.PHONY: mqtt-build
mqtt-build: ## Build mqtt-generator Docker image
	cd applications/mqtt-generator && docker build -t mqtt-generator:latest .

.PHONY: mqtt-import
mqtt-import: mqtt-build ## Build and import mqtt-generator to k3d
	k3d image import mqtt-generator:latest --cluster homelab-local

.PHONY: mqtt-deploy
mqtt-deploy: ## Deploy mqtt-generator
	kubectl apply -k gitops/apps/local/mqtt-generator

.PHONY: mqtt-logs
mqtt-logs: ## Watch mqtt-generator logs
	kubectl logs -f deployment/mqtt-generator -n mqtt-generator

.PHONY: mqtt-subscribe
mqtt-subscribe: ## Subscribe to MQTT topics (requires mosquitto_sub and port-forward)
	@echo "Subscribing to homelab/# topics..."
	@echo "Make sure to run 'make port-mosquitto' in another terminal first"
	mosquitto_sub -h localhost -t 'homelab/#' -v

##@ Development Helpers

.PHONY: lint
lint: ## Lint Kubernetes manifests (requires kubeconform)
	@if command -v kubeconform >/dev/null 2>&1; then \
		find gitops -name '*.yaml' -type f | xargs kubeconform -summary; \
	else \
		echo "kubeconform not installed. Install with: brew install kubeconform"; \
	fi

.PHONY: validate
validate: ## Validate Flux resources
	flux check --pre
	find gitops/infrastructure gitops/apps -name '*.yaml' -type f | xargs -I {} flux validate {}

.PHONY: diff
diff: ## Show diff between local and cluster (requires kubectl-diff)
	@echo "Infrastructure diff:"
	kubectl diff -k gitops/clusters/local/
	@echo "\nApps diff:"
	kubectl diff -k gitops/apps/local/

##@ Utilities

.PHONY: clean
clean: ## Clean up Docker resources
	docker system prune -af --volumes

.PHONY: context-local
context-local: ## Switch kubectl context to local cluster
	kubectl config use-context k3d-homelab-local

.PHONY: context-prod
context-prod: ## Switch kubectl context to production
	@echo "WARNING: Switching to production cluster!"
	@read -p "Are you sure? [y/N] " -n 1 -r; \
	echo; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		kubectl config use-context homelab; \
	fi

.PHONY: setup-tools
setup-tools: ## Install required development tools (macOS only)
	@if [ "$$(uname)" = "Darwin" ]; then \
		brew install kubectl k3d fluxcd/tap/flux helm kubeconform; \
	else \
		echo "This target only works on macOS. Please install tools manually."; \
		echo "See docs/LOCAL_DEVELOPMENT.md for instructions."; \
	fi
