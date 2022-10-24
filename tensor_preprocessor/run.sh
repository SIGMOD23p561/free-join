# Use: bash run.sh <queries> <database>
# Example: bash run.sh join-order-benchmark imdb

original_queries="../queries/original/$1"
preprocessed_queries="../queries/tensor_preprocessed/$1"

mkdir $preprocessed_queries/counts
mkdir $preprocessed_queries/filters
mkdir $preprocessed_queries/joins
mkdir $preprocessed_queries/data

# Run every query through the preprocessor to get its separated filters and joins
# ls -I does not work on Mac OS X
for query in `ls $original_queries`
do
	./target/release/tensor_preprocessor $original_queries/$query counts "${query%.*}" > $preprocessed_queries/counts/$query
	./target/release/tensor_preprocessor $original_queries/$query filters "${query%.*}" > $preprocessed_queries/filters/$query
	./target/release/tensor_preprocessor $original_queries/$query joins "${query%.*}" > $preprocessed_queries/joins/$query
done

cd $preprocessed_queries
bash generate_filter_tables.sh $2
