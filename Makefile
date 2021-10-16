.PHONY:  check_gopath  git_head  install dependencies  clean uninstall purge

EXE=xkcdget.legacy

test: ./test.sh install
	./test.sh $(GOPATH)/bin/$(EXE) || exit 1

install: check_gopath git_head dependencies $(EXE)
	# We can't use go install, because it would overwrite the xkcdget binary.
	# Apparently it names the resulting executable after the package (the
	# directory name), not after the binary produced by go build.
	mv -f $(EXE) "$(GOPATH)/bin/"

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
