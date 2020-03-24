.PHONY : tests build

help:  ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

##@ Testing

feature-check: ## build features in commmon combination to be sure it all stays together
	cargo check --all-features
	cargo check --no-default-features
	cargo check

unit-test: ## Run all unit tests
	cargo test --examples

tests: feature-check unit-test ## Run all tests we have

bench: ## Run criterion based benchmark, works on stable Rust
	cargo bench

