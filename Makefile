.PHONY: build test lint clean install help

BINARY_NAME=naru

help: ## Show this help message
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build the project in release mode
	cargo build --release

test: ## Run all tests
	cargo test

lint: ## Run clippy and check formatting
	cargo clippy -- -D warnings
	cargo fmt --all -- --check

clean: ## Clean build artifacts
	cargo clean

install: build ## Build and install the binary to /usr/local/bin
	sudo cp target/release/$(BINARY_NAME) /usr/local/bin/

uninstall: ## Remove the binary from /usr/local/bin
	sudo rm /usr/local/bin/$(BINARY_NAME)

dev: ## Run the project in development mode
	cargo run --

