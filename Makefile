.PHONY:  gopath  git_head  install dependencies  clean uninstall purge

EXE=xkcdget.legacy

install: git_head gopath dependencies $(EXE)
	# We can't use go install, because it would overwrite the xkcdget binary.
	# Apparently it names the resulting executable after the package (the
	# directory name), not after the binary produced by go build.
	mv -f $(EXE) "$(GOPATH)/bin/"

git_head:
	printf "package main\nfunc buildCommit() string { return \"$$(git describe --all --long --dirty)\" }" > $@.go

$(EXE): $(wildcard *.go)
	if [ -x /usr/bin/goimports ]; then goimports -w "$<"; fi
	#Don't run go get, because that installs xkcdget.legacy as xkcdget
	go build -o "$(EXE)"

dependencies: "$(GOPATH)/src/github.com/majewsky/pwget"
"$(GOPATH)/src/github.com/majewsky/pwget": gopath
	go get -u "github.com/majewsky/pwget"
	cd $@; make # the path is already in "" so we don't need them here


clean:
	rm -f "$(EXE)" git_head.go

uninstall: gopath
	rm -f "$(GOPATH)/bin/$(EXE)"

purge: uninstall clean


gopath:
	if [ -z "$(GOPATH)" ]; then exit 1; fi
