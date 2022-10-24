for File in `ls ./third_party/imdb/data/${F%.*}/`
do
	echo "COPY ${File%.*} FROM './third_party/imdb/data/$File' (escape '\');"
done
