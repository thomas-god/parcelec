locations=parcelec tests

install:
	poetry install -v --no-root
install-no-dev:
	poetry install -v --no-dev --no-root
format:
	poetry run isort $(locations)
	poetry run black $(locations)
typecheck:
	poetry run mypy $(locations)
lint:
	poetry run pylint --ignore-imports=yes $(locations)
test:
	poetry run pytest --disable-warnings -vv --doctest-modules tests
test-docker:
	export DOCKER_BUILDKIT=1
	export COMPOSE_DOCKER_CLI_BUILD=1
	docker-compose down && docker-compose up --force-recreate --build --abort-on-container-exit