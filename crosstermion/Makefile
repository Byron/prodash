.PHONY : tests build

help:  ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

##@ Testing

check: ## build features in commmon combination to be sure it all stays together
	#if cargo check; then false; fi
	cargo check --features crossterm
	cargo check --features termion
	cargo check --features crossterm,tui
	cargo check --features termion,tui
	cargo check --features crossterm,tui,tui/termion,tui-react
	cargo check --features termion,tui,tui/crossterm,tui-react
	cargo check --all-features


