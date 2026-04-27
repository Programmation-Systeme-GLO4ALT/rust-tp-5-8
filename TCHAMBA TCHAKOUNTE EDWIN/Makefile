.PHONY: all build-all clean-all run-tp5 run-tp6 test-tp5 test-tp6 test-all

TP5_DIR = tp5-parseur_fichier
TP6_DIR = tp6-abstraction_generique

all: build-all

# --- Compilation -------------------------------------------------------------
build-all:
	cd $(TP5_DIR) && cargo build
	cd $(TP6_DIR) && cargo build

# --- Exécution ---------------------------------------------------------------
run-tp5:
	cd $(TP5_DIR) && cargo run

run-tp6:
	cd $(TP6_DIR) && cargo run

# --- Tests -------------------------------------------------------------------
test-tp5:
	cd $(TP5_DIR) && cargo test

test-tp6:
	cd $(TP6_DIR) && cargo test

test-all: test-tp5 test-tp6

# --- Nettoyage ---------------------------------------------------------------
clean-all:
	cd $(TP5_DIR) && cargo clean
	cd $(TP6_DIR) && cargo clean
	rm -f $(TP6_DIR)/sortie_fusion.txt
