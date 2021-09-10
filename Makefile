.PHONY: clean run

LIBNAME=skedge
EXE=$(LIBNAME)_demo
SRC=./examples/ffi/c/main.c
RUSTBUILD=cargo build
RUSTFLAGS=--release --features ffi
CC=gcc
FLAGS=-std=c11 -Wall -Werror -pedantic
LD_PATH=./target/release
SO = lib$(LIBNAME).so
SO_PATH=$(LD_PATH)/$(SO)
LD=-L $(LD_PATH) -l $(LIBNAME)

$(EXE): $(SO_PATH)
	$(CC) $(FLAGS) $(LD) $(SRC) -o $(EXE)

$(SO_PATH):
	$(RUSTBUILD) $(RUSTFLAGS)

clean:
	@rm -r $(SO_PATH)
	@rm -r $(EXE)

run: clean $(EXE)
	LD_LIBRARY_PATH=$(LD_PATH) ./$(EXE)