# Use: bash test.sh <queries> <database>
# Example: bash test.sh join-order-benchmark imdb

original_queries="../queries/original/$1"
preprocessed_queries="../queries/preprocessed/$1"

plain_database="../data/$2/$2_plain.db"
main_database="../data/$2/$2.db"

# Appends each query's test to temp.sql
for query in `ls $original_queries`
do
	# Run the original benchmark query
	# echo "CREATE TABLE original_query AS "
	# cat "$original_queries/$query"

	# Load the filtered tables from the preprocessed parquet
	for preprocessed_table in `ls $preprocessed_queries/data/${query%.*}/`
	do
		echo "DROP TABLE IF EXISTS ${preprocessed_table%.*};"
		echo "CREATE TABLE ${preprocessed_table%.*} AS SELECT * FROM '$preprocessed_queries/data/${query%.*}/$preprocessed_table';"
	done

	# Run the preprocessed join query using the filtered tables
	# echo "CREATE TABLE preprocessed_query AS "
	echo "PRAGMA enable_profiling=json;"
	echo "PRAGMA profiling_output='../logs/${query%.*}.json';"
	cat "$preprocessed_queries/joins/$query"
	echo "PRAGMA disable_profiling;"

	# Get the difference between the original and preprocessed query
	echo ".print 'Testing $query'"
	# echo "SELECT * FROM original_query EXCEPT SELECT * FROM preprocessed_query;" 
	# echo "DROP TABLE IF EXISTS original_query;"
	# echo "DROP TABLE IF EXISTS preprocessed_query;"

done > temp.sql

# Runs all tests in temp.sql
cp $plain_database $main_database
../duckdb/build/release/duckdb << EOF
.open '$main_database'
.read temp.sql
EOF

# rm temp.sql
