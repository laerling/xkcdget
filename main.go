package main

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"log"
	"os"
	"os/exec"

	"github.com/tilinna/z85"
)

func main() {
	// find pwget
	pwgetExe, err := exec.LookPath("pwget2")
	failOnError(err, "Cannot find pwget on your system")

	pwgetCmd := exec.Command(pwgetExe, os.Args[1:]...)
	pwgetCmd.Stdin = os.Stdin
	pwgetCmd.Stderr = os.Stderr
	key, err := pwgetCmd.Output()
	failOnError(err, "Could not run pwget")

	// print passphrase
	fmt.Printf("%s_%s_%s_%s\n",
		string(chooseWordFromList(adjectivesList, key[0:9])),
		string(chooseWordFromList(nounsList, key[10:19])),
		string(chooseWordFromList(verbsList, key[20:29])),
		string(chooseWordFromList(nounsList, key[30:39])))
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
