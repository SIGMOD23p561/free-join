# there are also some extra files extracted from the tarball,
# but this is enough to work
CSVS= \
    aka_name.csv \
	aka_title.csv \
	cast_info.csv \
	char_name.csv \
	company_name.csv \
	company_type.csv \
	comp_cast_type.csv \
	complete_cast.csv \
	info_type.csv \
	keyword.csv \
	kind_type.csv \
	link_type.csv \
	movie_companies.csv \
	movie_info.csv \
	movie_info_idx.csv \
	movie_keyword.csv \
	movie_link.csv \
	name.csv \
	person_info.csv \
	role_type.csv \
	title.csv

all: $(CSVS) imdb_plain.db

imdb_plain.db: import.sh $(IMDB)
	rm -f $@
	bash $<

# don't delete the csvs, they take a long time to extract
.PRECIOUS: $(CSVS)

$(CSVS):
	wget -nc http://homepages.cwi.nl/~boncz/job/imdb.tgz
	tar xvf imdb.tgz

clean:
	rm -f *.db

clean_all:
	rm -f *.db
	rm -f *.csv
	rm -f *.parquet
	rm load.sql
	rm schema.sql
	rm -f schematext.sql
