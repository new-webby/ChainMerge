.PHONY: help test test-core test-api build-web run-api make-api run api run-web dev docker-up docker-down docker-logs

help:
	@echo "Available targets:"
	@echo "  make test         - Run core and API tests"
	@echo "  make test-core    - Run core tests"
	@echo "  make test-api     - Run API tests"
	@echo "  make build-web    - Build web app"
	@echo "  make run-api      - Run Rust API locally"
	@echo "  make make-api     - Alias for run-api"
	@echo "  make run api      - Alias form for run-api"
	@echo "  make run-web      - Run web app locally"
	@echo "  make dev          - Show local dev startup commands"
	@echo "  make docker-up    - Build and run full stack with Docker"
	@echo "  make docker-down  - Stop Docker stack"
	@echo "  make docker-logs  - Tail Docker logs"

test: test-core test-api

test-core:
	cd core/chaincodec && cargo test

test-api:
	cd services/api && cargo test

build-web:
	cd apps/web && npm run build

run-api:
	cd services/api && cargo run

make-api: run-api

run:
	@:

api: run-api

run-web:
	cd apps/web && npm run dev

dev:
	@echo "Terminal 1: make run-api"
	@echo "Terminal 2: make run-web"
	@echo "Open: http://127.0.0.1:5173"

docker-up:
	docker compose up --build

docker-down:
	docker compose down

docker-logs:
	docker compose logs -f
