.PHONY: install clean uninstall purge

EXE=xget

$(EXE): $(wildcard *.go)
	go build -o $(EXE)

install: $(EXE)
	install -m 0755 $(EXE) /bin/$(EXE)

clean:
	rm -f $(EXE)

uninstall:
	rm -f /bin/$(EXE)

purge: uninstall clean
