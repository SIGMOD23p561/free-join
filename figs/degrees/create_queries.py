import csv

def main():
    with open('join_attributes.csv', newline='') as file:
        join_reader = csv.reader(file, delimiter=',', quotechar='|')
        for row in join_reader:
            table = row[0]
            attribute = row[1]
            pair_name = table + "_" + attribute
            query = open("./queries/" + pair_name + ".sql", "w")

            query_body = "SELECT COUNT(*) FROM " + table + " GROUP BY " + attribute
            query_total = "COPY (" + query_body + ") TO './tables/" + pair_name + ".csv' (HEADER, DELIMITER ',');"

            query.write(query_total)
            query.close()

if __name__ == "__main__":
    main()