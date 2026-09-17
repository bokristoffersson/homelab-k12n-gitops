# Homelab GitOps Makefile
# Convenience commands for cluster management

.PHONY: help
help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

.PHONY: dev-watch
dev-watch: ## Watch all pods in all namespaces
	kubectl get pods -A --watch

##@ Flux Commands

.PHONY: flux-check
flux-check: ## Check Flux prerequisites and status
	flux check

.PHONY: flux-reconcile
flux-reconcile: ## Reconcile Flux kustomizations
	flux reconcile source git flux-system
	flux reconcile kustomization flux-system

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

##@ Redpanda

.PHONY: redpanda-list
redpanda-list: ## List Redpanda topics
	kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk topic list

.PHONY: redpanda-info
redpanda-info: ## Show Redpanda cluster info
	kubectl exec -n redpanda-v2 redpanda-v2-0 -- rpk cluster info

.PHONY: redpanda-consume
redpanda-consume: ## Consume from a topic (use TOPIC=name)
	kubectl exec -it redpanda-v2-0 -n redpanda-v2 -- \
		rpk topic consume $(TOPIC) --num 10

##@ MQTT

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

##@ Utilities

.PHONY: clean
clean: ## Clean up Docker resources
	docker system prune -af --volumes

.PHONY: setup-tools
setup-tools: ## Install required development tools (macOS only)
	@if [ "$$(uname)" = "Darwin" ]; then \
		brew install kubectl fluxcd/tap/flux helm kubeconform; \
	else \
		echo "This target only works on macOS. Please install tools manually."; \
	fi
