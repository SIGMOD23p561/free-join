plain_database="../../data/imdb/imdb_plain.db"
main_database="../../data/imdb/imdb.db"

cp $plain_database $main_database

for query in `ls queries`
do
../../duckdb/build/release/duckdb << EOF
.open '$main_database'
.read queries/$query
EOF
done
