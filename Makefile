lint:
		cargo clippy --all --all-targets --all-features --no-deps -- --deny warnings
		cargo fmt --all -- --check
