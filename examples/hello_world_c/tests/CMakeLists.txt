cmake_minimum_required(VERSION 3.20)
project(hello_world_c_tests LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include(FetchContent)
FetchContent_Declare(
  googletest
  GIT_REPOSITORY https://github.com/google/googletest.git
  GIT_TAG        v1.15.2
)
FetchContent_MakeAvailable(googletest)

enable_testing()

add_executable(hello_world_c_tests main_test.cpp)

target_include_directories(hello_world_c_tests PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../bindings)
target_link_libraries(hello_world_c_tests PRIVATE GTest::gtest_main)

if(UNIX AND NOT APPLE)
    target_link_libraries(hello_world_c_tests PRIVATE dl)
endif()

include(GoogleTest)
gtest_discover_tests(hello_world_c_tests)
