.PHONY: install clean

EXE=xget

$(EXE): $(wildcard *.go)
	go build -o $(EXE)

install: $(EXE)
	install -m 0755 $(EXE) /bin/$(EXE)

clean:
	rm -f $(EXE)
	rm -f /bin/$(EXE)
