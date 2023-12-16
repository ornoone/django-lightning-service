
help:
	@grep -P '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'



install:
	test -d .venv || pipx run virtualenv .venv
	.venv/bin/pip install -r requirements/requirements.txt

pip-compile:
	pipx run --spec pip-tools pip-compile requirements/requirements.in