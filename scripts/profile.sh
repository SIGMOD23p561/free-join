set -v 

top_dir=${PWD}
log_plan_dir="${top_dir}/logs/plan-profiles/"
duckdb_binary="${top_dir}/duckdb/build/release/duckdb"
profile_sql="${top_dir}/scripts/profile.sql"

mkdir -p ${log_plan_dir}

cd ${log_plan_dir}
GJ_TABLE=1 ${duckdb_binary} -c ".read ${profile_sql}"
cd ${top_dir}
