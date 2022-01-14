cmake_minimum_required(VERSION 3.10)

project(wigner_benchmarks)

include(ExternalProject)

ExternalProject_Add(wigxjpf
    URL http://fy.chalmers.se/subatom/wigxjpf/wigxjpf-1.11.tar.gz
    CONFIGURE_COMMAND ""
    BUILD_COMMAND make -C ../wigxjpf/
    INSTALL_COMMAND ""
)

install(FILES
    ${CMAKE_CURRENT_BINARY_DIR}/wigxjpf-prefix/src/wigxjpf/lib/libwigxjpf.a
    DESTINATION "lib"
)
