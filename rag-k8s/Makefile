.PHONY: venv deps build-index run-agent test clean

PYTHON := python3
VENV := venv
VENV_BIN := $(VENV)/bin

venv:
	$(PYTHON) -m venv $(VENV)
	@echo "Virtual environment created. Activate with: source $(VENV)/bin/activate"

deps: venv
	$(VENV_BIN)/pip install --upgrade pip
	$(VENV_BIN)/pip install -e .
	@echo "Dependencies installed"

build-index: deps
	$(VENV_BIN)/python index/build_index.py
	@echo "Index built successfully"

run-agent: deps
	$(VENV_BIN)/python -m agent.tool
	@echo "Agent ready"

test: deps
	$(VENV_BIN)/pytest tests/ -q

clean:
	rm -rf $(VENV)
	rm -rf build/ dist/ *.egg-info
	rm -f index/faiss.index index/meta.json
	rm -rf logs/*.log
	find . -type d -name __pycache__ -exec rm -rf {} +
	find . -type f -name '*.pyc' -delete
