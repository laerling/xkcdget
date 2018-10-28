.PHONY: gopath install dependencies clean uninstall purge

EXE=xkcdget

$(EXE): $(wildcard *.go) dependencies
	if [ -x /usr/bin/goimports ]; then goimports -w "$<"; fi
	go get
	go build -o "$(EXE)"
	go install

dependencies: "$(GOPATH)/src/github.com/majewsky/pwget"

"$(GOPATH)/src/github.com/majewsky/pwget": gopath
	go get "github.com/majewsky/pwget"
	cd "$@"; make

install: /bin/"$(EXE)"

/bin/"$(EXE)":
	sudo -E install -m 0755 "$(GOPATH)/bin/$(EXE)" $@

clean:
	rm -f "$(EXE)"

uninstall:
	sudo -E rm -f "/bin/$(EXE)"
	# gopath is no dep of this target because empty gopath is effectively the same as /bin/$(EXE)
	sudo -E rm -f "$(GOPATH)/bin/$(EXE)"

purge: uninstall clean

gopath:
	if [ -z "$(GOPATH)" ]; then exit 1; fi
