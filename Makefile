.PHONY:  check_gopath  git_head  install dependencies  clean uninstall purge

EXE=xkcdget

install: check_gopath git_head dependencies $(EXE)
	go install

git_head:
	printf "package main\nfunc buildCommit() string { return \"$$(git describe --all --long --dirty)\" }" > $@.go

$(EXE): $(wildcard *.go)
	if [ -x /usr/bin/goimports ]; then goimports -w "$<"; fi
	go get
	go build -o "$(EXE)"

dependencies: "$(GOPATH)/src/github.com/majewsky/pwget"
"$(GOPATH)/src/github.com/majewsky/pwget": check_gopath
	go get -u "github.com/majewsky/pwget"
	cd $@; make # the path is already in "" so we don't need them here


clean:
	rm -f "$(EXE)" git_head.go

uninstall: check_gopath
	rm -f "$(GOPATH)/bin/$(EXE)"

purge: uninstall clean


check_gopath:
	if [ -z "$(GOPATH)" ]; then exit 1; fi
