% Generated by roxygen2: do not edit by hand
% Please edit documentation in R/relational.R
\name{rel_project}
\alias{rel_project}
\title{Lazily project a DuckDB relation object}
\usage{
rel_project(rel, exprs)
}
\arguments{
\item{rel}{the DuckDB relation object}

\item{exprs}{a list of DuckDB expressions to project}
}
\value{
the now projected \code{duckdb_relation} object
}
\description{
Lazily project a DuckDB relation object
}
\examples{
con <- DBI::dbConnect(duckdb())
rel <- rel_from_df(con, mtcars)
rel2 <- rel_project(rel, list(expr_reference("cyl"), expr_reference("disp")))
}
