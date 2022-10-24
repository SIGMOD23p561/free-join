#!/usr/bin/env python3

import json
import statistics
import itertools

import plotly.express as px
import plotly.graph_objects as go

# import matplotlib
# matplotlib.rcParams['pdf.fonttype'] = 42
# matplotlib.rcParams['ps.fonttype'] = 42
# matplotlib.rcParams['savefig.bbox'] = 'tight'


def plot(data):
    # get the sorted query names
    ddb = sorted(data['duckdb'], key=lambda x: x['time'])
    indexes = {d['query']: i for i, d in enumerate(ddb)}
    gj = sorted(data['gj'], key=lambda x: indexes[x['query']])

    # preprocess all data
    for d in itertools.chain(ddb, gj):
        d['query'] = d['query'].lstrip('IMDBQ')

    fig1 = px.line(
        ddb, x='query', y='time',
    )

    # preprocess gj data
    for d in gj:
        d['total'] = statistics.mean(d['time'])
        d['optimize'] = 'O{}'.format(d['optimize'])

    fig2 = px.line(
        gj, x='query', y='total', color='optimize',
    )

    fig = go.Figure(data=fig1.data + fig2.data)

    fig.update_layout(
        title='IMDB Query Times',
        yaxis_title='Time (s)',
    )
    fig.write_html('plot.html')


if __name__ == '__main__':
    import sys

    with open(sys.argv[1]) as f:
        data = json.load(f)

    plot(data)
