EXE=xkcdget
INSTALL_DIR=~/bin

.PHONY: $(EXE) test install clean uninstall purge

test: ./test.sh target/release/$(EXE)
	./test.sh target/release/$(EXE) || exit 1

install: target/release/$(EXE)
	mkdir -p ~/bin
	cp $< ~/bin/

target/release/$(EXE): $(wildcard src/*.rs)
	./version-update-reminder.sh
	cargo build --release

clean:
	rm -rf target

uninstall:
	rm -f $(INSTALL_DIR)/$(EXE)

purge: uninstall clean
