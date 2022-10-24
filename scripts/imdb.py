#!/usr/bin/env python3

import json
import numpy as np
import matplotlib.pyplot as plt

plt.rcParams['pdf.fonttype'] = 42
plt.rcParams['ps.fonttype'] = 42
plt.rcParams['savefig.bbox'] = 'tight'

colors = ['#377eb8', '#ff7f00', '#4daf4a', '#f781bf', '#a65628']


def plot(data):

    ddb = data['duckdb']
    ddb = {d['query']: d['time'] for d in ddb}

    gj = data['gj']

    ablate_opt = {}
    ablate_trie = {}
    ablate_vec = {}
    gj_e2e = {}

    for i, record in enumerate(gj):
        j = i % 11
        match j:
            case 0 | 1 | 2:
                if ablate_opt.get(j) is None:
                    ablate_opt[j] = {}
                ablate_opt[j][record['query']] = record['time'][0]
            case 3 | 4 | 5:
                if ablate_trie.get(j) is None:
                    ablate_trie[j] = {}
                ablate_trie[j][record['query']] = record['time'][0]
            case 6 | 7 | 8 | 9:
                if ablate_vec.get(j) is None:
                    ablate_vec[j] = {}
                ablate_vec[j][record['query']] = record['time'][0]
            case 10:
                if gj_e2e.get(j) is None:
                    gj_e2e[j] = {}
                gj_e2e[j][record['query']] = record['time'][0]

    # e2e
    fig, ax = plt.subplots()
    plt.xscale('log')
    plt.yscale('log')

    x = ddb.values()
    y = ablate_opt[1].values()
    z = gj_e2e[10].values()
    ax.scatter(x, y, s=5, color='black', label='Free Join')
    ax.scatter(x, z, s=5, color='silver', label='Generic Join')

    lims = [
        np.min([ax.get_xlim(), ax.get_ylim()]),  # min of both axes
        np.max([ax.get_xlim(), ax.get_ylim()]),  # max of both axes
    ]

    ax.plot(lims, lims, color='gray', linewidth=0.5)
    ax.set_aspect('equal')
    ax.set_xlabel('Binary Join time (s)')
    ax.set_ylabel('Free Join / Generic Join time (s)')
    ax.set_xlim(lims)
    ax.set_ylim(lims)

    plt.legend(loc='upper left')
    plt.savefig('e2e.pdf', format='pdf')

    # merging ablation
    fig, ax = plt.subplots()
    plt.xscale('log')
    plt.yscale('log')

    y0 = ablate_opt[0].values()
    y1 = ablate_opt[1].values()
    y2 = ablate_opt[2].values()
    ax.scatter(y0, y2, s=5, color='silver', label='occurence merging')
    ax.scatter(y0, y1, s=5, color='black', label='variable merging')

    lims = [
        np.min([ax.get_xlim(), ax.get_ylim()]),  # min of both axes
        np.max([ax.get_xlim(), ax.get_ylim()]),  # max of both axes
    ]

    ax.plot(lims, lims, color='gray', linewidth=0.5)
    ax.set_aspect('equal')
    ax.set_xlabel('time w/ basic merging (s)')
    ax.set_ylabel('time w/ further merging (s)')
    ax.set_xlim(lims)
    ax.set_ylim(lims)

    plt.legend(loc='upper left')
    plt.savefig('merge.pdf', format='pdf')

    # trie ablation
    fig, ax = plt.subplots()
    plt.xscale('log')
    plt.yscale('log')

    y0 = ablate_trie[3].values()
    y1 = ablate_trie[4].values()
    y2 = ablate_trie[5].values()
    ax.scatter(y0, y1, s=5, color='silver', label='SLT')
    ax.scatter(y0, y2, s=5, color='black', label='COLT')

    lims = [
        np.min([ax.get_xlim(), ax.get_ylim()]),  # min of both axes
        np.max([ax.get_xlim(), ax.get_ylim()]),  # max of both axes
    ]

    ax.plot(lims, lims, color='gray', linewidth=0.5)
    ax.set_aspect('equal')
    ax.set_xlabel('time w/ simple trie (s)')
    ax.set_ylabel('time w/ lazy tries (s)')
    ax.set_xlim(lims)
    ax.set_ylim(lims)

    plt.legend(loc='upper left')
    plt.savefig('trie.pdf', format='pdf')

    # vector ablation
    fig, ax = plt.subplots()
    plt.xscale('log')
    plt.yscale('log')

    y0 = ablate_vec[6].values()
    y1 = ablate_vec[7].values()
    y2 = ablate_vec[8].values()
    y3 = ablate_vec[9].values()

    ax.scatter(y0, y1, s=5, color='lightgray', label='batch 10x')
    ax.scatter(y0, y2, s=5, color='silver', label='batch 100x')
    ax.scatter(y0, y3, s=5, color='black', label='batch 1000x')

    lims = [
        np.min([ax.get_xlim(), ax.get_ylim()]),  # min of both axes
        np.max([ax.get_xlim(), ax.get_ylim()]),  # max of both axes
    ]

    ax.plot(lims, lims, color='gray', linewidth=0.5)
    ax.set_aspect('equal')
    ax.set_xlabel('time w/o vectorization (s)')
    ax.set_ylabel('time w/ vectorization (s)')
    ax.set_xlim(lims)
    ax.set_ylim(lims)

    plt.legend(loc='upper left')
    plt.savefig('vec.pdf', format='pdf')


if __name__ == '__main__':
    import sys

    with open(sys.argv[1]) as f:
        data = json.load(f)

    plot(data)
