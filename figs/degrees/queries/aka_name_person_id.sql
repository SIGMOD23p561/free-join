COPY (SELECT COUNT(*) FROM aka_name GROUP BY person_id) TO './tables/aka_name_person_id.csv' (HEADER, DELIMITER ',');