COPY (SELECT COUNT(*) FROM cast_info GROUP BY role_id) TO './tables/cast_info_role_id.csv' (HEADER, DELIMITER ',');