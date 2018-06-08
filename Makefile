.PHONY: install clean uninstall purge

EXE=xkcdget

$(EXE): $(wildcard *.go)
	if [ -x /usr/bin/goimports ]; then goimports -w $<; fi
	go get
	go build -o $(EXE)

install: /bin/$(EXE)

/bin/$(EXE):
	install -m 0755 $(EXE) /bin/$(EXE)

clean:
	rm -f $(EXE)

uninstall:
	rm -f /bin/$(EXE)

purge: uninstall clean
