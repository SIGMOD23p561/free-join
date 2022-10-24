import sqlite3


SCHEMA_PATH = 'imdb/schematext.sql'

# simple script to load IMDB table schemas into an in-memory sqlite3 instance
con = sqlite3.connect(':memory:')
cur = con.cursor()

with open(SCHEMA_PATH, mode='r') as schema_file:
    create_table_statements = schema_file.read().split('\n\n')
    for create_table_statement in create_table_statements:
        cur.execute(create_table_statement)

cur.execute("SELECT name FROM sqlite_master WHERE type='table';")
results = cur.fetchall()
print(results)
con.commit()
con.close()
