.PHONY: test test-full lint licence oracle veto unsafe-feature bench bench2 fmt clippy doc

test: lint licence
	cargo test

test-full: test
	cargo test -- --ignored
	$(MAKE) unsafe-feature
	$(MAKE) veto

licence:
	scripts/licence.sh

lint:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt

clippy:
	cargo clippy --all-targets -- -D warnings

doc:
	cargo doc --no-deps

unsafe-feature:
	scripts/unsafe_feature_gate.sh

oracle:
	scripts/build_oracle.sh

veto:
	@test -x /tmp/470M1-oracle/provari/target/release/provari || \
		{ echo "missing oracle: run make oracle" >&2; exit 1; }
	scripts/veto.sh

bench:
	python3 bench/run_bench.py \
		--provari /tmp/470M1-oracle/provari/target/release/provari \
		--corpus bench/corpus.json \
		--out docs/plans/parity/measurements/plan1/results.json

bench2:
	cargo build --release
	python3 docs/plans/parity/measurements/plan2/run_bench.py \
		--provari /tmp/470M1-oracle/provari/target/release/provari \
		--candidate target/release/470MI \
		--out docs/plans/parity/measurements/plan2/results.json
