//===----------------------------------------------------------------------===//
//                         DuckDB
//
// duckdb/common/types/vector_cache.hpp
//
//
//===----------------------------------------------------------------------===//

#pragma once

#include "duckdb/common/types.hpp"
#include "duckdb/common/vector.hpp"
#include "duckdb/common/types/vector_buffer.hpp"

namespace duckdb {
class Vector;

//! The VectorCache holds cached data that allows for re-use of the same memory by vectors
class VectorCache {
public:
	//! Instantiate a vector cache with the given type
	explicit VectorCache(const LogicalType &type);

	buffer_ptr<VectorBuffer> buffer;

public:
	void ResetFromCache(Vector &result) const;

	const LogicalType &GetType() const;
};

} // namespace duckdb
