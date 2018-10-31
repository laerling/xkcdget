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
	# We use the fifth line of `git log` for the commit message, because .git/COMMIT_EDITMSG is unreliable:
	# It does not show merge commits and it's empty if the last `git commit` was aborted
	echo -e "package main\nfunc buildCommit() string {" \
		"return \"$$(git describe --all --long)\"" \
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
