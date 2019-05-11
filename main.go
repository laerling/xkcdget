package main

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path"
	"strings"

	"github.com/tilinna/z85"
)

func main() {
	// print commit of build
	// write to stderr so that it doesn't get piped into xsel
	os.Stderr.Write([]byte("xkcdget build: " + buildCommit() + "\n"))

	// find pwget
	pwgetName := "pwget"
	pwgetExe, err := exec.LookPath(pwgetName)
	failOnError(err, "Finding "+pwgetName+" on your system failed")

	// check revocation list
	revList := path.Join(os.Getenv("HOME"), ".pwget2-revocation")
	if _, err = os.Stat(revList); err != nil {
		// warn user (on stderr so it doesn't get piped into xsel)
		os.Stderr.Write([]byte("Warning: Revocation list missing or not readable" +
			" (expected in " + revList + ")\n"))
	}

	// call pwget
	pwgetCmd := exec.Command(pwgetExe, os.Args[1:]...)
	pwgetCmd.Stdin = os.Stdin
	pwgetCmd.Stderr = os.Stderr
	key, err := pwgetCmd.Output()
	failOnError(err, "Running pwget failed")

	// print passphrase
	fmt.Printf("%s%s%s%s_1",
		strings.Title(string(chooseWordFromList(list, key[0:10]))),
		strings.Title(string(chooseWordFromList(list, key[10:20]))),
		strings.Title(string(chooseWordFromList(list, key[20:30]))),
		strings.Title(string(chooseWordFromList(list, key[30:40]))))

	//write the newline on stderr only, so that it is not copied when
	//piping stdout to xsel or xclip
	os.Stderr.Write([]byte("\n"))
}

//failOnError prints MSG when ERR is not nil
func failOnError(err error, msg string) {
	if err != nil {
		fmt.Println(msg)
		log.Fatal(err)
	}
}

//chooseWordFromList chooses a word from LIST, determined by SEED
func chooseWordFromList(list []string, seed []byte) string {
	// decode 10 bytes (encoding 64 bits)
	var scalarSeed [8]byte
	_, err := z85.Decode(scalarSeed[:], seed)
	failOnError(err, "Cannot decode slice of key")

	// read decoded bytes into uint64
	var intSeed uint64
	binary.Read(bytes.NewReader(scalarSeed[:]), binary.LittleEndian, &intSeed)

	return list[intSeed%uint64(len(list))]
}
