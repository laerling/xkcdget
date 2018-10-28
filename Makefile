.PHONY:  gopath  install dependencies  clean uninstall purge

EXE=xkcdget

install: gopath dependencies $(EXE)
	go install

$(EXE): $(wildcard *.go)
	if [ -x /usr/bin/goimports ]; then goimports -w "$<"; fi
	go get
	go build -o "$(EXE)"

dependencies: "$(GOPATH)/src/github.com/majewsky/pwget"
"$(GOPATH)/src/github.com/majewsky/pwget": gopath
	go get "github.com/majewsky/pwget"
	cd $@; make # the path is already in "" so we don't need them here


clean:
	rm -f "$(EXE)"

uninstall: gopath
	rm -f "$(GOPATH)/bin/$(EXE)"

purge: uninstall clean


gopath:
	if [ -z "$(GOPATH)" ]; then exit 1; fi
