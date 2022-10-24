.PHONY: all clean clean_all

DUCKDB=duckdb/build/release/duckdb
PREPROCESSOR=preprocessor/target/release/preprocessor
IMDB=data/imdb/imdb_plain.db
IMDB_PAR=data/imdb/name.parquet
DATA=queries/preprocessed/join-order-benchmark/data
PROFILE=logs/plan-profiles
IMDB_JSON_NAMES=$(shell for i in $$(seq 113); do printf 'IMDBQ%03d.json\n' $$i; done)
IMDB_JSONS=$(addprefix logs/plan-profiles/,$(IMDB_JSON_NAMES))

all: $(DATA)

$(DUCKDB): duckdb/src
	BUILD_BENCHMARK=1 $(MAKE) -C duckdb -j


$(PREPROCESSOR):
	cd preprocessor && cargo build --release

$(IMDB): duckdb
	$(MAKE) -C data/imdb

${PROFILE}: ${DUCKDB} ${IMDB} scripts/profile.sh scripts/profile.sql
	bash scripts/profile.sh

.PHONY: imdb_jsons
imdb_jsons: $(IMDB_JSONS)
	echo $(IMDB_JSONS)
	echo $(IMDB_JSON_NAMES)

.PRECIOUS: $(IMDB_JSONS)
$(IMDB_JSONS) &: $(DUCKDB) $(IMDB)
	(cd duckdb && \
	 GJ_TABLE=1 build/release/benchmark/benchmark_runner --threads=1 'IMDBQ.*' && \
	 mv IMDB*.json ../logs/plan-profiles/)


$(IMDB_PAR): $(DUCKDB) scripts/transform.sql
	$(DUCKDB) -c ".read './scripts/transform.sql'"

$(DATA): preprocessor/run.sh $(DUCKDB) $(IMDB) $(IMDB_PAR) ${PROFILE} $(PREPROCESSOR)
	cd preprocessor && bash run.sh join-order-benchmark imdb && touch ../$@

test: preprocessor/test.sh $(DATA)
	cd preprocessor && bash test.sh join-order-benchmark imdb

GJ_SRC=$(shell find gj/src -name "*.rs")

gj/gj.json: $(GJ_SRC)
	# (cd gj && time cargo run --profile=release-final -- -O0 -O1 -O2 -n5 --json=gj.json)
	(cd gj && time cargo run --profile=release-final -- -O1 -n1 --json=gj.json)

plot.html: ./scripts/plot.py gj/gj.json
	$^

clean_all: clean
	rm -rf duckdb && mkdir duckdb
	$(MAKE) -C data/imdb clean_all
	cd queries/preprocessed/join-order-benchmark \
	&& rm -f -d -r filters \
	&& rm -f -d -r joins \
	&& rm -f -d -r data \
	&& cd - \
	&& rm -f -d -r logs/plan-profiles \
	&& rm -f -d -r logs/scan-profiles

clean:
	$(MAKE) -C duckdb clean
	$(MAKE) -C data/imdb clean
	cd preprocessor && cargo clean
