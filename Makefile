.PHONY: venv
venv:
	uv venv
	. .venv/bin/activate && uv pip install -r requirements-test.txt -r requirements-dev.txt -r requirements-docs.txt

.PHONY: clean
clean:
	rm -rf .venv
	rm -rf .pytest_cache
	rm -rf .hypothesis
	rm -rf .mypy_cache
	rm -rf .ruff_cache
	
.PHONY: lint
lint:
	ruff check .
	ruff format .
	isort .
	mypy .

.PHONY: test
test:
	pytest -n auto

.PHONY: build
build:
	maturin build

.PHONY: install
install:
	maturin develop

.PHONY: build-docs
build-docs:
	mkdocs build