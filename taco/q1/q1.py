import csv
import pytaco as pt

# This is an example translation of a SQL query to a TACO query.
# The query is a simplified version of q1.sql from JOB, where
# we have removed the filters, and changed the MIN aggregate to COUNT
# 
# SELECT COUNT(*)
#   FROM ct, it, mc, miidx, t
#  WHERE ct.id = mc.company_type_id
#    AND t.id = mc.movie_id
#    AND t.id = mi_idx.movie_id
#    AND mc.movie_id = mi_idx.movie_id
#    AND it.id = mi_idx.info_type_id;

# Loading data; each dimension to be joined must have the same size.
# We can take the max ID to be the size of the dimension, 
# or, it may be OK to just use the MAX_INT for every dimension.
t = pt.tensor([100])
miidx = pt.tensor([100, 50, 30])
mc = pt.tensor([100])
it = pt.tensor([50])
ct = pt.tensor([30])
q = pt.tensor()

# We need to generate the csv files so that they only contain 
# join attributes for the given query, for example t only has id
with open('t.csv') as file:
  rdr = csv.reader(file, delimiter=',') 
  for row in rdr:
    # treat each row as tensor index
    coo = [int(n) for n in row]
    t.insert(coo, 1.0)

with open('miidx.csv') as file:
  rdr = csv.reader(file, delimiter=',') 
  for row in rdr:
    coo = [int(n) for n in row]
    miidx.insert(coo, 1.0)

with open('mc.csv') as file:
  rdr = csv.reader(file, delimiter=',') 
  for row in rdr:
    coo = [int(n) for n in row]
    mc.insert(coo, 1.0)

with open('it.csv') as file:
  rdr = csv.reader(file, delimiter=',') 
  for row in rdr:
    coo = [int(n) for n in row]
    it.insert(coo, 1.0)

with open('ct.csv') as file:
  rdr = csv.reader(file, delimiter=',') 
  for row in rdr:
    coo = [int(n) for n in row]
    ct.insert(coo, 1.0)

# GJ plan:
# [ t.id = miidx.movie_id = mc.movie_id
# , it.id = mi_idx.info_type_id
# , mc.company_type_id = ct.id ]

tid, itid, ctid = pt.get_index_vars(3)

q[None] = t[tid] * miidx[tid, itid, ctid] * mc[tid] * it[itid] * ct[ctid]
q.compile()
q.assemble()

# Packing builds the trie for each tensor
t.pack(); miidx.pack(); mc.pack(); it.pack(); ct.pack()
q.compute()

print(q)
