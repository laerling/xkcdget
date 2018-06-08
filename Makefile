.PHONY: install dependencies clean uninstall purge

EXE=xkcdget

$(EXE): $(wildcard *.go) dependencies
	if [ -x /usr/bin/goimports ]; then goimports -w "$<"; fi
	go get
	go build -o "$(EXE)"
	go install

dependencies: "$(GOPATH)/src/github.com/majewsky/pwget"

"$(GOPATH)/src/github.com/majewsky/pwget":
	go get "github.com/majewsky/pwget"
	cd "$@"; make

install: /bin/"$(EXE)"

/bin/"$(EXE)":
	install -m 0755 "$(EXE)" /bin/"$(EXE)"

clean:
	rm -f "$(EXE)"

uninstall:
	rm -f "/bin/$(EXE)"
	rm -f "$(GOPATH)/bin/$(EXE)"

purge: uninstall clean
