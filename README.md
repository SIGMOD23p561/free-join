# Free Join

This is the artifact for [Free Join: Unifying Worst-Case Optimal and Traditional Joins](https://arxiv.org/abs/2301.10841). 
Due to time constraints, we provide instructions to run only one set of experiments. 

# Requirements

1. Standard linux tools, like `make`, `wget` etc. Please watch for errors and install any missing binaries. 
2. Rust, installed with https://rustup.rs
3. Any dependencies required to build [DuckDB](https://github.com/duckdb/duckdb#development), including [CMake](https://cmake.org), Python3 and a `C++11` compliant compiler. We include DuckDB's source code in this repository, with minimal modifiactions to produce machine-readable query plans. The Makefile builds DuckDB. 
4. For plotting, install plotly and pandas (e.g. with pip/pip3). 

# Building and Running Free Join
Running `make` after installing the dependencies will build the code and prepare the benchmark data.
The Makefile downloads the IMDB dataset from a third-party repository and may fail sometimes, 
so it may be necessary to rerun `make`. 
If the download keeps failing, please obtain the IMDB dataset elsewhere, and place the tar ball 
at the appropriate path according to the Makefile.

Then, execute `make plot.html` to run the experiments. The results are plotted in plot.html which can be opened in a browser. 

# The Code
Our Rust implementation of Free Join is found under `gj`. 
