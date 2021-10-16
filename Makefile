.PHONY:  check_gopath  git_head  install dependencies  clean uninstall purge

EXE=xkcdget

test: ./test.sh install
	./test.sh $(GOPATH)/bin/$(EXE) || exit 1

install: check_gopath git_head dependencies $(EXE)
	go install

git_head:
	printf "package main\nfunc buildCommit() string { return \"$$(git describe --all --long --dirty)\" }" > $@.go

$(EXE): $(wildcard *.go)
	if [ -x /usr/bin/goimports ]; then goimports -w "$<"; fi
	go mod tidy
	go build -o "$(EXE)"

dependencies:
"$(GOPATH)/src/github.com/majewsky/pwget": check_gopath
	go install "github.com/majewsky/pwget"@latest
	cd $@; make # the path is already in "" so we don't need them here

clean:
	rm -f "$(EXE)" git_head.go

uninstall: check_gopath
	rm -f "$(GOPATH)/bin/$(EXE)"

purge: uninstall clean


check_gopath:
	if [ -z "$(GOPATH)" ]; then exit 1; fi
