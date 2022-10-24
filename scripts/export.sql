COPY (SELECT id FROM company_type AS ct WHERE ct.kind = 'production companies') TO '../data/ct.parquet' (format PARQUET);
COPY (SELECT id FROM info_type AS it WHERE it.info = 'rating') TO '../data/it.parquet' (format PARQUET);
COPY (SELECT id FROM kind_type AS kt WHERE kt.kind = 'movie') TO '../data/kt.parquet' (format PARQUET);
copy (select mi.movie_id, mi.info_type_id, mi.info from movie_info AS mi) TO '../data/mi_full.parquet' (format PARQUET);
copy (select miidx.movie_id, miidx.info_type_id, miidx.info from movie_info_idx AS miidx) TO '../data/miidx_full.parquet' (format PARQUET);
copy (select id, t.kind_id, t.title from title AS t) TO '../data/t_full.parquet' (format PARQUET);
copy (select mc.movie_id, mc.company_type_id, mc.company_id from movie_companies AS mc) to '../data/mc.parquet' (format PARQUET);
