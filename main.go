package main

import (
	"fmt"
	"io/ioutil"
	"log"
	"os"
)

func main() {
	// read key from stdin
	input, err := ioutil.ReadAll(os.Stdin)
	checkErr(err, "Cannot read key from stdin")
	key := input[:len(input)-1]
	if len(key) < 39 {
		log.Fatal("The length of the key must be at least 40 bytes")
	}

	// print passphrase
	fmt.Printf("%s_%s_%s_%s\n",
		string(chooseWordFromList(adjectivesList, key[0:9])),
		string(chooseWordFromList(nounsList, key[10:19])),
		string(chooseWordFromList(verbsList, key[20:29])),
		string(chooseWordFromList(nounsList, key[30:39])))
}

//checkErr prints MSG when ERR is not nil
func checkErr(err error, msg string) {
	if err != nil {
		fmt.Println(msg)
		log.Fatal(err)
	}
}

//chooseWordFromList chooses a word from LIST, determined by SEED
func chooseWordFromList(list []string, seed []byte) string {
	seedScalar := 0
	for _, b := range seed {
		seedScalar += int(b) //TODO evaluate this
	}
	return list[seedScalar%len(list)]
}
