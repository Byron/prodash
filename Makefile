.PHONY : tests build

help:  ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

##@ Testing

clippy: ## Run cargo-clippy
	cargo clippy --all-features

check: ## build features in commmon combination to be sure it all stays together
	cargo check --all-features
	cargo check --no-default-features
	cargo check --features render-tui,render-tui-crossterm
	cargo check --features render-tui,render-tui-termion
	cargo check --features render-line,render-line-termion
	cargo check --features render-line,render-line-termion,render-line-autoconfigure
	cargo check --features render-line,render-line-termion,render-line-autoconfigure,signal-hook
	cargo check --features render-line,render-line-crossterm
	cargo check --features render-line,render-line-termion,render-tui,render-tui-termion --example dashboard-termion
	cargo check --features render-line,render-line-crossterm,render-tui,render-tui-crossterm,signal-hook,render-line-autoconfigure --example dashboard
	cargo check --features unit-bytes,unit-duration,unit-human,render-tui,render-tui-crossterm,render-line,render-line-crossterm,signal-hook --example units
	cargo check

unit-test: ## Run all unit tests
	cargo test --features unit-bytes,unit-human,unit-duration

tests: clippy check unit-test ## Run all tests we have

bench: ## Run criterion based benchmark, works on stable Rust
	cargo bench

##@ Development

fmt: ## run nightly rustfmt for its extra features, but check that it won't upset stable rustfmt
	cargo +nightly fmt --all -- --config-path rustfmt-nightly.toml
	cargo +stable fmt --all -- --check


