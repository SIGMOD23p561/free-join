//===----------------------------------------------------------------------===//
//                         DuckDB
//
// duckdb/parser/parsed_data/drop_info.hpp
//
//
//===----------------------------------------------------------------------===//

#pragma once

#include "duckdb/parser/parsed_data/parse_info.hpp"
#include "duckdb/common/enums/catalog_type.hpp"

namespace duckdb {

struct DropInfo : public ParseInfo {
	DropInfo() : schema(INVALID_SCHEMA), if_exists(false), cascade(false) {
	}

	//! The catalog type to drop
	CatalogType type;
	//! Schema name to drop from, if any
	string schema;
	//! Element name to drop
	string name;
	//! Ignore if the entry does not exist instead of failing
	bool if_exists = false;
	//! Cascade drop (drop all dependents instead of throwing an error if there
	//! are any)
	bool cascade = false;

public:
	unique_ptr<DropInfo> Copy() const {
		auto result = make_unique<DropInfo>();
		result->type = type;
		result->schema = schema;
		result->name = name;
		result->if_exists = if_exists;
		result->cascade = cascade;
		return result;
	}
};

} // namespace duckdb
