#!/bin/sh

if [ "$#" -eq 1 ] && [ -f "$1" ]; then
  input_filename=$1
  name=$(basename "$input_filename" ".csv")
  parentdir=$(dirname "$input_filename")  
  output_filename="$parentdir""/""$name"".tns"
elif [ "$#" -eq 2 ] && [ -f "$1" ]; then
  input_filename=$1
  output_filename=$2
else
  echo "Usage: $0 input_filename output_filename" >&2
  exit 1
fi

col_num=$(head -n 1 "$input_filename" | tr ',' '\n' | wc -l)

sort_args=""
for i in `seq 1 $col_num`
do
	sort_args="${sort_args} -k$i,$i"
done

cat $input_filename | sed 's/,/ /g' | sed "s/$/ 1.0/" | sort -n $sort_args > $output_filename
