
CC ?= cc
CFLAGS ?= -O2 -std=c11 -Wall -Wextra -pedantic
LDLIBS ?= -lm -lcrypto -lgmp

EXAMPLES := 	collatz_1000 	control_system 	deep_taxonomy_100000 	delfour 	euler_identity 	fibonacci 	goldbach_1000 	gps 	kaprekar_6174 	matrix_mechanics 	odrl_dpv_ehds_risk_ranked 	path_discovery 	pn_junction_tunneling 	polynomial 	transistor_switch 	sudoku

BINARIES := $(EXAMPLES:%=bin/%)

all: $(BINARIES)

bin/%: examples/%.c | bin
	$(CC) $(CFLAGS) $< -o $@ $(LDLIBS)

bin:
	mkdir -p bin

check: $(BINARIES)
	@bash tools/check_with_timing.sh

clean:
	rm -rf bin

.PHONY: all check clean
