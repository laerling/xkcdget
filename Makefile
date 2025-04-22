EXE=xkcdget_legacy
EXE_INSTALL=xkcdget.legacy
INSTALL_DIR=~/bin

.PHONY: $(EXE) test install clean uninstall purge

target/release/$(EXE): $(wildcard src/*.rs)
	./version-update-reminder.sh
	cargo build --release

test: ./test.sh target/release/$(EXE)
	./test.sh target/release/$(EXE) || exit 1

install: target/release/$(EXE)
	mkdir -p ~/bin
	cp $< ~/bin/$(EXE_INSTALL)

clean:
	rm -rf target

uninstall:
	rm -f $(INSTALL_DIR)/$(EXE)

purge: uninstall clean
