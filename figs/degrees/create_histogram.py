import sys
import csv
import matplotlib.pyplot as plt

def count_elements(seq):
    hist = {}
    for i in seq:
        hist[int(i[0])] = hist.get(int(i[0]), 0) + 1
    return hist

def plot_hist(hist, image_name):
    plt.rcParams["figure.figsize"] = [10.00, 5.00]
    plt.rcParams["figure.autolayout"] = True

    values = []
    frequencies = []
    for item in hist:
        values.append(item)
        frequencies.append(hist[item])

    plt.bar(values, frequencies)

    plt.savefig("./histograms/" + image_name)

def main():
    file_name = sys.argv[1]
    image_name = file_name[8:-4] + ".png"
    with open(file_name, newline='') as file:
        query_reader = csv.reader(file, delimiter=',', quotechar='|')
        next(query_reader)
        hist = count_elements(query_reader)
        plot_hist(hist, image_name)

if __name__ == "__main__":
    main()
