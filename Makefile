.PHONY: all dev release production clean

APP = miniping

# Default: dev build (macOS)
all: dev

# --- Dev build (fast iteration) ---
dev:
	cargo build
	@echo "\nBinary: target/debug/$(APP)"

# --- Release build (balanced) ---
release:
	cargo build --release
	@echo "\nBinary: target/release/$(APP)"

# --- Linux release (cross-compile to musl) ---
linux-release:
	cargo build --release --target x86_64-unknown-linux-musl
	cp target/x86_64-unknown-linux-musl/release/$(APP) $(APP)-linux-x86_64-release
	@echo "\nBinary: $(APP)-linux-x86_64-release"

# --- Linux production (maximum size reduction) ---
linux-production:
	cargo build --profile production --target x86_64-unknown-linux-musl
	cp target/x86_64-unknown-linux-musl/production/$(APP) $(APP)-linux-x86_64-production
	cp $(APP)-linux-x86_64-production $(APP)-linux-x86_64
	@echo "\nBinary: $(APP)-linux-x86_64-production  →  $(APP)-linux-x86_64"

# --- Build all variants ---
all-variants: dev release linux-release linux-production
	@echo "\n=== All builds complete ==="
	@ls -lh $(APP)-linux-x86_64* target/release/$(APP) 2>/dev/null || true

clean:
	cargo clean
	rm -f $(APP)-linux-x86_64 $(APP)-linux-x86_64-release $(APP)-linux-x86_64-production $(APP)-macos