ifeq ($(shell uname),Darwin)
    EXT := dylib
else
    EXT := so
endif

all: target/debug/libdouble_input.$(EXT)
	g++ -std=c++11 src/main.cpp -L ./target/debug/ -lagent_test -o run
	LD_LIBRARY_PATH=./target/debug/ ./run

target/debug/libdouble_input.$(EXT): src/lib.rs Cargo.toml
	cargo build

clean:
	rm -rf target
	rm -rf run
