.PHONY: test test-full lint licence oracle veto unsafe-feature bench bench-goal \
	bench-sets bench-calibrate bench-guard tablebases fmt clippy doc

ORACLE ?= /tmp/470M1-oracle
PROVARI := $(ORACLE)/provari/target/release/provari
TABLEBASES := $(ORACLE)/tablebases/3-4-5
M1 := docs/plans/foundation/measurements/plan1
# The per-host thread count chosen by calibration; part of every goal row.
N ?= 1

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

tablebases:
	scripts/fetch_tablebases.sh

veto:
	@test -x $(PROVARI) || \
		{ echo "missing oracle: run make oracle" >&2; exit 1; }
	scripts/veto.sh

# Selection precedes any candidate run; it writes bench/sets/*.json.
bench-sets:
	python3 bench/run_bench.py --mode select --column matched-v1 \
		--set-out bench/sets/matched-v1.json --guide model --threads 1 \
		--out $(M1)/select-matched
	python3 bench/run_bench.py --mode select --column goal-v1 \
		--set-out bench/sets/goal-v1.json --guide model --threads $(N) \
		--tablebases $(TABLEBASES) --out $(M1)/select-goal

bench-calibrate:
	python3 bench/run_bench.py --mode calibrate --guide model \
		--tablebases $(TABLEBASES) --out $(M1)/calibration

# The matched column: reference model guide, one thread, no tablebases.
bench:
	cargo build --release
	python3 bench/run_bench.py --mode row --set bench/sets/matched-v1.json \
		--guide model --threads 1 --claim --out $(M1)/matched

# The goal column: reference plus N threads and the 3-4-5 tablebases.
bench-goal:
	cargo build --release
	python3 bench/run_bench.py --mode row --set bench/sets/goal-v1.json \
		--guide model --threads $(N) --tablebases $(TABLEBASES) --claim \
		--out $(M1)/goal

# The hand guard column: three historical cells, both sides hand, one thread.
bench-guard:
	cargo build --release
	python3 bench/run_bench.py --mode row --set bench/sets/guard.json \
		--guide hand --threads 1 --claim --out $(M1)/guard
