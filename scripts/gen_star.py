import csv

N = 10000
k = 3

names = ['sr', 'ss', 'st', 'su', 'sv', 'sw']


def r(n):
    al = n % k
    ar = (n + 1) % k
    l = [(al, i) for i in range(1, N//2 + 1)]
    r = [(ar, i) for i in range(N//2 + 1, N + 1)]

    return [(-1, -1)] + l + r


rels = [r(n) for n in range(k)]

for (i, rel) in enumerate(rels):
    with open(names[i] + '.csv', 'w') as f:
        writer = csv.writer(f)
        writer.writerows(rel)
