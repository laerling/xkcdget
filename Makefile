.PHONY:  test install dependencies  clean uninstall purge

EXE=xkcdget

test: ./test.sh install
	./test.sh $(GOPATH)/bin/$(EXE) || exit 1

install: dependencies $(EXE)
	go install

$(EXE): $(wildcard *.go)
	if [ -x /usr/bin/goimports ]; then goimports -w "$<"; fi
	go mod tidy
	go build -o "$(EXE)"

dependencies:
"$(GOPATH)/src/github.com/majewsky/pwget":
	go install "github.com/majewsky/pwget"@latest
	cd $@; make # the path is already in "" so we don't need them here

clean:
	rm -f "$(EXE)"

uninstall:
	rm -f "$(GOPATH)/bin/$(EXE)"

purge: uninstall clean
