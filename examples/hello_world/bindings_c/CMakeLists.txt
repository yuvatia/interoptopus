cmake_minimum_required(VERSION 3.20)
project(hello_world_c_test LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Path to Rust cdylib output (passed via -DRUST_LIB_DIR=...)
if(NOT DEFINED RUST_LIB_DIR)
    message(FATAL_ERROR "RUST_LIB_DIR must be set to the directory containing the Rust cdylib")
endif()

include(FetchContent)
FetchContent_Declare(
  googletest
  GIT_REPOSITORY https://github.com/google/googletest.git
  GIT_TAG        v1.15.2
)
FetchContent_MakeAvailable(googletest)

enable_testing()

add_executable(hello_world_c_test main_test.cpp)

target_include_directories(hello_world_c_test PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(hello_world_c_test PRIVATE GTest::gtest_main)

if(UNIX AND NOT APPLE)
    target_link_libraries(hello_world_c_test PRIVATE dl)
endif()

include(GoogleTest)
gtest_discover_tests(hello_world_c_test
    PROPERTIES ENVIRONMENT "LD_LIBRARY_PATH=${RUST_LIB_DIR}"
    DISCOVERY_MODE PRE_TEST
)
