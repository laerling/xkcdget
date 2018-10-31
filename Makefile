.PHONY:  gopath  clean_index git_head  install dependencies  clean uninstall purge

EXE=xkcdget

install: clean_index git_head gopath dependencies $(EXE)
	go install

clean_index:
	@if ! (git status|grep -q "working tree clean"); then \
		echo "Working tree not clean. Commit changes before building!" 1>&2; \
		exit 1; \
	fi;

git_head: .git/HEAD
	echo -e "package main\nfunc buildCommit() string {" \
		"return \"$(shell grep -o '[^/]\+$$' $<) at" \
		"$(shell git log|head -c15):" \
		"$(shell head -1 .git/COMMIT_EDITMSG)\"" \
		"}" > $@.go

$(EXE): $(wildcard *.go)
	if [ -x /usr/bin/goimports ]; then goimports -w "$<"; fi
	go get
	go build -o "$(EXE)"

dependencies: "$(GOPATH)/src/github.com/majewsky/pwget"
"$(GOPATH)/src/github.com/majewsky/pwget": gopath
	go get "github.com/majewsky/pwget"
	cd $@; make # the path is already in "" so we don't need them here


clean:
	rm -f "$(EXE)" git_head.go

uninstall: gopath
	rm -f "$(GOPATH)/bin/$(EXE)"

purge: uninstall clean


gopath:
	if [ -z "$(GOPATH)" ]; then exit 1; fi
