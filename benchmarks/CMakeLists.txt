cmake_minimum_required(VERSION 3.10)

project(wigner_benchmarks)

set(CMAKE_BUILD_TYPE "release" CACHE STRING "" FORCE)

include(ExternalProject)

ExternalProject_Add(wigxjpf
    URL http://fy.chalmers.se/subatom/wigxjpf/wigxjpf-1.11.tar.gz
    CONFIGURE_COMMAND ""
    BUILD_COMMAND make -C ../wigxjpf/
    INSTALL_COMMAND ""
    DOWNLOAD_EXTRACT_TIMESTAMP ON
)

install(FILES
    ${CMAKE_CURRENT_BINARY_DIR}/wigxjpf-prefix/src/wigxjpf/lib/libwigxjpf.a
    DESTINATION "lib"
)

file(DOWNLOAD
    "https://raw.githubusercontent.com/0382/WignerSymbol/f8c8dce9ceefda20ac4ea57bf36304b418e8c0c0/WignerSymbolSingleton.hpp"
    ${CMAKE_CURRENT_BINARY_DIR}/WignerSymbolSingleton.hpp
)

add_library(wigner-symbol-0382 STATIC wigner-symbol-0382.cpp)
target_include_directories(wigner-symbol-0382 PRIVATE ${CMAKE_CURRENT_BINARY_DIR})
target_compile_features(wigner-symbol-0382 PRIVATE cxx_std_17)
target_link_options(wigner-symbol-0382 PRIVATE -static-libc++)

install(TARGETS wigner-symbol-0382
    DESTINATION "lib"
)


# Get default C++ libraries to link
file(WRITE ${PROJECT_BINARY_DIR}/cxx_link_libs.cmake "")
foreach(lib ${CMAKE_CXX_IMPLICIT_LINK_LIBRARIES})
    file(APPEND ${PROJECT_BINARY_DIR}/cxx_link_libs.cmake "${lib}\n")
endforeach()

# Get default C++ search path for libraries
file(WRITE ${PROJECT_BINARY_DIR}/cxx_link_dirs.cmake "")
foreach(dir ${CMAKE_CXX_IMPLICIT_LINK_DIRECTORIES})
    file(APPEND ${PROJECT_BINARY_DIR}/cxx_link_dirs.cmake "${dir}\n")
endforeach()
