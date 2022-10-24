create table n (n integer);
copy n from 'a.csv';
create table r(x integer, y integer);
create table s(y integer, z integer);
create table t(x integer, z integer);
insert into r select distinct n1.n as x, n2.n as y from n as n1, n as n2 where n1.n=0 or n2.n=0;
insert into s select distinct n1.n as y, n2.n as z from n as n1, n as n2 where n1.n=0 or n2.n=0;
insert into t select distinct n1.n as x, n2.n as z from n as n1, n as n2 where n1.n=0 or n2.n=0;
PRAGMA enable_profiling=json;
PRAGMA profiling_output='triangle.json';
select r.x, s.y, t.z from r, s, t where r.x = t.x and r.y = s.y and s.z = t.z;
