.PHONY: clean deps run

LIBNAME=skedge
EXE=$(LIBNAME)_demo
SRC=./examples/ffi/c/main.c
RUSTBUILD=cargo build
RUSTFLAGS=--release
CC=gcc
FLAGS=-std=c11 -Wall -Werror -pedantic
LD_PATH=./target/release
SO = lib$(LIBNAME).so
SO_PATH=$(LD_PATH)/$(SO)
LD=-L $(LD_PATH) -l $(LIBNAME)

$(EXE): deps
	$(CC) $(FLAGS) $(LD) $(SRC) -o $(EXE)

deps: $(SO_PATH)

$(SO_PATH):
	$(RUSTBUILD) $(RUSTFLAGS)

clean:
	cargo clean
	rm $(EXE)

run: $(EXE)
	LD_LIBRARY_PATH=$(LD_PATH) ./$(EXE)