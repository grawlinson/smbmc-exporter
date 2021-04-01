.DEFAULT: help
.PHONY: help clean clean-pyc clean-build dist

help: ## Display this help section
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z0-9_-]+:.*?## / {printf "\033[36m%-38s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

clean: clean-pyc clean-build ## Delete all artifacts

clean-pyc: ## Delete Python cache artifacts
	@find . -name '*.pyc' -delete
	@find . -name '*.pyo' -delete
	@find . -name __pycache__ -delete

clean-build: ## Delete distribution artifacts
	@rm --force --recursive build dist src/*.egg-info docs/_build

dist: clean ## Generate distribution artifacts
	poetry build

