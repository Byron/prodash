.PHONY : tests build

help:  ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

##@ Testing

check: ## build features in commmon combination to be sure it all stays together
	#if cargo check; then false; fi
	cargo check --features crossterm
	cargo check --features termion
	cargo check --features crossterm,input-async
	cargo check --features termion,input-async
	cargo check --features crossterm,tui
	cargo check --features termion,tui
	cargo check --features tui-react-termion
	cargo check --features tui-react-termion,input-async
	cargo check --features tui-react-crossterm
	cargo check --features tui-react-crossterm,input-async
	cargo check --features tui-termion
	cargo check --features tui-crossterm
	cargo check --features color
	cargo check --all-features


