#!/usr/bin/env python3

import json
import matplotlib
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.patches import Ellipse
import matplotlib.transforms as transforms

plt.rcParams['pdf.fonttype'] = 42
plt.rcParams['ps.fonttype'] = 42
plt.rcParams['savefig.bbox'] = 'tight'

colors = ['#377eb8', '#ff7f00', '#4daf4a', '#f781bf', '#a65628']


def confidence_ellipse(x, y, ax, n_std=3.0, facecolor='none', **kwargs):
    """
    Create a plot of the covariance confidence ellipse of *x* and *y*.

    Parameters
    ----------
    x, y : array-like, shape (n, )
        Input data.

    ax : matplotlib.axes.Axes
        The axes object to draw the ellipse into.

    n_std : float
        The number of standard deviations to determine the ellipse's radiuses.

    **kwargs
        Forwarded to `~matplotlib.patches.Ellipse`

    Returns
    -------
    matplotlib.patches.Ellipse
    """
    # if x.size != y.size:
    #     raise ValueError("x and y must be the same size")

    x = np.exp(x)
    y = np.exp(y)

    cov = np.cov(x, y)
    pearson = cov[0, 1]/np.sqrt(cov[0, 0] * cov[1, 1])
    # Using a special case to obtain the eigenvalues of this
    # two-dimensional dataset.
    ell_radius_x = np.sqrt(1 + pearson)
    ell_radius_y = np.sqrt(1 - pearson)
    ellipse = Ellipse((0, 0), width=ell_radius_x * 2, height=ell_radius_y * 2,
                      facecolor=facecolor, **kwargs)

    # Calculating the standard deviation of x from
    # the squareroot of the variance and multiplying
    # with the given number of standard deviations.
    scale_x = np.sqrt(cov[0, 0]) * n_std
    mean_x = np.mean(x)

    # calculating the standard deviation of y ...
    scale_y = np.sqrt(cov[1, 1]) * n_std
    mean_y = np.mean(y)

    transf = transforms.Affine2D() \
        .rotate_deg(45) \
        .scale(scale_x, scale_y) \
        .translate(mean_x, mean_y)

    ellipse.set_transform(transf + ax.transData)
    return ax.add_patch(ellipse)


def plot(good_data, bad_data):

    ddb_good = good_data['duckdb']
    ddb_bad = bad_data['duckdb']
    gj_good = good_data['gj']
    gj_bad = bad_data['gj']

    ddb_good = {d['query']: d['time'] for d in ddb_good}
    ddb_bad = {d['query']: d['time'] for d in ddb_bad}
    fj_good = {d['query']: d['time'][0] for d in gj_good if d['optimize'] == 1}
    fj_bad = {d['query']: d['time'][0] for d in gj_bad if d['optimize'] == 1}
    gj_good = {d['query']: d['time'][0] for d in gj_good if d['optimize'] == 0}
    gj_bad = {d['query']: d['time'][0] for d in gj_bad if d['optimize'] == 0}

    # ddb_slowdown = [ddb_bad[q] / ddb_good[q] for q in ddb_good]
    # fj_slowdown = [fj_bad[q][0] / fj_good[q][0] for q in fj_good]
    # gj_slowdown = [gj_bad[q][0] / gj_good[q][0] for q in gj_good]

    fig, ax = plt.subplots()
    plt.xscale('log')
    plt.yscale('log')

    ax.scatter(ddb_bad.values(), gj_bad.values(), s=5,
               color='silver', label='Generic Join')
    ax.scatter(ddb_bad.values(), fj_bad.values(), s=5,
               color='black', label='Free Join')

    # ax.scatter(ddb_good.values(), ddb_bad.values(), s=5,
    #            color='black', label='DuckDB')
    #            # color='#377eb8', label='DuckDB')
    # ax.scatter(fj_good.values(), fj_bad.values(), s=5,
    #            color='black', label='Free Join')
    #            # color='#ff7f00', marker='2', label='Free Join')
    # ax.scatter(gj_bad.values(), gj_good.values(), s=5,
    #            color='black', label='Generic Join')
    #            # color='#4daf4a', label='Generic Join')
    # ax.scatter(ddb_slowdown, gj_slowdown, s=5,
    #            color='silver', label='Generic Join')

    # confidence_ellipse(list(gj_good.values()), list(gj_bad.values()), ax,
    #                    alpha=0.5, facecolor='pink', edgecolor='purple', zorder=0)

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
    plt.savefig('robust.pdf', format='pdf')


if __name__ == '__main__':
    import sys

    with open(sys.argv[1]) as f:
        good_data = json.load(f)
    with open(sys.argv[2])as f:
        bad_data = json.load(f)

    plot(good_data, bad_data)
