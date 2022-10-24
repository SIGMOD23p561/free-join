CREATE TABLE sr (
  x integer NOT NULL,
  y integer NOT NULL
);

CREATE TABLE ss (
  x integer NOT NULL,
  y integer NOT NULL
);

CREATE TABLE st (
  x integer NOT NULL,
  y integer NOT NULL
);

COPY sr FROM 'star/s3/sr.csv';
COPY ss FROM 'star/s3/ss.csv';
COPY st FROM 'star/s3/st.csv';

SET threads TO 1;
PRAGMA enable_profiling=json;
PRAGMA profiling_output='./s3.json';

SELECT *
  FROM sr, ss, st
 WHERE sr.x = ss.x
   AND ss.x = st.x
   AND sr.x = st.x;
