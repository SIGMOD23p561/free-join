SELECT MIN(mi.info) AS release_date, MIN(miidx.info) AS rating, MIN(t.title) AS german_movie
FROM read_parquet('../data/cn.parquet') as cn, 
read_parquet('../data/mi_full.parquet') as mi, 
read_parquet('../data/miidx_full.parquet') as miidx, 
read_parquet('../data/ct.parquet') as ct, 
read_parquet('../data/t_full.parquet') as t, 
read_parquet('../data/it2.parquet') as it2, 
read_parquet('../data/kt.parquet') as kt, 
read_parquet('../data/mc.parquet') as mc, 
read_parquet('../data/it.parquet') as it, 
WHERE mi.movie_id = t.id
AND it2.id = mi.info_type_id
AND kt.id = t.kind_id
AND mc.movie_id = t.id
AND cn.id = mc.company_id
AND ct.id = mc.company_type_id
AND miidx.movie_id = t.id
AND it.id = miidx.info_type_id
AND mi.movie_id = miidx.movie_id
AND mi.movie_id = mc.movie_id
AND miidx.movie_id = mc.movie_id
;
