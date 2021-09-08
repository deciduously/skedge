.PHONY: clean demo deps so run

LIBNAME=skedge
EXE=$(LIBNAME)_demo
SRC=./examples/ffi/c/main.c
RUSTBUILD=cargo build
RUSTFLAGS=--release
CC=gcc
FLAGS=-std=c11 -Wall -Werror -pedantic
LD_PATH=./target/release
LD=-L $(LD_PATH) -l $(LIBNAME)

demo: deps
	$(CC) $(FLAGS) $(LD) $(SRC) -o $(EXE)

deps: so

so:
	$(RUSTBUILD) $(RUSTFLAGS)

clean:
	rm $(EXE)

run: demo
	LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$(LD_PATH) ./$(EXE)