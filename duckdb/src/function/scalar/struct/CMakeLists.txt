add_library_unity(duckdb_func_struct OBJECT struct_extract.cpp struct_pack.cpp)
set(ALL_OBJECT_FILES
    ${ALL_OBJECT_FILES} $<TARGET_OBJECTS:duckdb_func_struct>
    PARENT_SCOPE)
