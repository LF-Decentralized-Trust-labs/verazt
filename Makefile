# Compiler version managers the frontends shell out to; each one installs
# the compiler versions a contract's pragma needs on demand. `vyper-select`
# is also needed for Vyper input, but it is not published on PyPI.
PYTHON_TOOLS := solc-select

.PHONY: deps
deps:
	@if command -v uv >/dev/null 2>&1; then \
		for tool in $(PYTHON_TOOLS); do uv tool install --upgrade $$tool || exit 1; done; \
	elif command -v pipx >/dev/null 2>&1; then \
		for tool in $(PYTHON_TOOLS); do pipx install $$tool || exit 1; done; \
	else \
		echo "error: install uv or pipx first" >&2; exit 1; \
	fi
