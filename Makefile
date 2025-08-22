EXEC 	:= ./target/debug/cli.exe
CARGO   := cargo

VAULT_PATH 	?= "C:/Users/onnia/Documents/notes/obisidian-notes-main"
VAULT_FLAG 	?= --vault "$(VAULT_PATH)"
VAULT_ENV	?= VAULT_PATH=""
NOTE_PATH 	?= "test_app/test.md"
BODY      	?= "Test"

.PHONY: all build

all: build

build:
	@cargo build

.PHONY: new append open show stats

new: build
	@$(EXEC) $(VAULT_FLAG) new "$(BODY)"

append: build
	@$(EXEC) $(VAULT_FLAG) append -n "$(NOTE_PATH)" "$(BODY)"

open: build
	@$(EXEC) $(VAULT_FLAG) open

show: build
	@$(EXEC) $(VAULT_FLAG) show -n "$(NOTE_PATH)"

stats: build
	@$(EXEC) $(VAULT_FLAG) stats