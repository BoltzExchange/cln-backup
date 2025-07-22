docker-build:
	docker buildx build . -o=build --target=binaries

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: docker-build clippy
