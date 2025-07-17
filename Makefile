PLUGIN_NAME := backup
CLN_CONTAINER := boltz-cln-2
PLUGIN_PATH := /root/.lightning/plugins/$(PLUGIN_NAME)

UNAME_S := $(shell uname -s)

.PHONY: build

build:
ifeq ($(UNAME_S),Darwin)
	CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-unknown-linux-gnu-gcc \
	CC_x86_64_unknown_linux_gnu=x86_64-unknown-linux-gnu-gcc \
	CXX_x86_64_unknown_linux_gnu=x86_64-unknown-linux-gnu-g++ \
	cargo build --target x86_64-unknown-linux-gnu
	mkdir -p target/debug
	cp target/x86_64-unknown-linux-gnu/debug/$(PLUGIN_NAME) target/debug/
else
	cargo build
endif

regtest-start: build
	git submodule init
	git submodule update
	chmod -R 777 regtest 2> /dev/null || true
	cd regtest && COMPOSE_PROFILES=ci ./start.sh
	docker exec $(CLN_CONTAINER) mkdir -p /root/.lightning/plugins
	docker cp target/debug/$(PLUGIN_NAME) $(CLN_CONTAINER):/root/.lightning/plugins/
	docker cp backup.toml $(CLN_CONTAINER):/root/.lightning/regtest/
	docker exec $(CLN_CONTAINER) chmod +x /root/.lightning/plugins/$(PLUGIN_NAME)
	docker exec $(CLN_CONTAINER) lightning-cli --regtest plugin start $(PLUGIN_PATH)

regtest-stop:
	cd regtest && ./stop.sh

clippy:
	cargo clippy --all-targets --all-features -- -D warnings
