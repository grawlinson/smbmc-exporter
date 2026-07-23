_default:
  just --list

[doc('generate wheel/sdist for distribution')]
build:
    uv build

[doc('Lint Python files with `ruff check`')]
check *ARGS:
    uv run ruff check {{ ARGS }}

[doc('Format Python files with `ruff format`')]
format *ARGS:
    uv run ruff format {{ ARGS }}

[doc('Remove all files ignored from git')]
clean:
  git clean -fX .
