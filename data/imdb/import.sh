../../duckdb/build/release/duckdb imdb_plain.db << EOF
.read schematext.sql
.read import.sql
.quit
EOF