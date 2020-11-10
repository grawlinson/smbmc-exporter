.DEFAULT: help
.PHONY: help clean clean-pyc clean-build dist

help:
	@echo "clean         : delete all artifacts"
	@echo "clean-pyc     : delete python cache artifacts"
	@echo "clean-build   : delete distribution artifacts"
	@echo "dist          : generate distribution artifacts"

clean: clean-pyc clean-build

clean-pyc:
	@find . -name '*.pyc' -delete
	@find . -name '*.pyo' -delete
	@find . -name __pycache__ -delete

clean-build:
	@rm --force --recursive build dist src/*.egg-info docs/_build

dist: clean
	poetry build

