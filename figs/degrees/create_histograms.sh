for table in `ls tables`
do
    python3 create_histogram.py ./tables/$table
done