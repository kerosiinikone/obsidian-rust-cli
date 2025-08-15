# Make the flags vars here, cleaner

build:
	cargo b

test_add_flag: build
	./target/debug/obsidian-rust-cli.exe --vault "C:\Users\onnia\Documents\notes\obisidian-notes-main" new "Hello" 

test_add_env: build
	VAULT_PATH="./" ./target/debug/obsidian-rust-cli.exe new "Hello" 

test_append_flag: build
	./target/debug/obsidian-rust-cli.exe --vault "C:\Users\onnia\Documents\notes\obisidian-notes-main" append -n "test_app\test.md" "Appended Test"