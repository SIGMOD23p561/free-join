# Use: bash generate_filter_tables.sh <database>
# Example: bash generate_filter_tables.sh imdb

plain_database="../../../data/$1/$1_plain.db"
main_database="../../../data/$1/$1.db"

for filter in `ls filters`
do
    cd data
    mkdir ${filter%.*}
    cd ..

    cd filters
    cat $filter
    cd ..
done > temp.sql

cp $plain_database $main_database
cd filters
../../../../duckdb/build/release/duckdb << EOF
.open '../$main_database'
.read '../temp.sql'
EOF
cd ..

rm temp.sql
