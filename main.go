package main

import (
	"bytes"
	"fmt"
	"io/ioutil"
	"log"
	"os"
)

func main() {
	// read lists
	nounsList := readList("nouns")
	adjectivesList := readList("adjectives")
	verbsList := readList("verbs")

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

//readList reads the file with name LISTNAME from the lists directory and
//returns it as a slice
func readList(listname string) []byte {
	listDump, err := ioutil.ReadFile("lists/" + listname)
	checkErr(err, "Cannot read file")
	return listDump
}

//getLine returns a slice containing the line (delimited by '\n') which
//contains the byte at position OFFSET in LIST
//FIXME Don't accept empty lines, rename to getNonEmptyLine
func getLine(list []byte, offset int) []byte {
	begin := bytes.LastIndexByte(list[:offset], '\n') + 1
	if begin < 0 {
		begin = 0
	}
	var relEnd = bytes.IndexByte(list[offset:], '\n')
	if relEnd < 0 {
		relEnd = len(list)
	}
	end := offset + relEnd
	return list[begin:end]
}

//chooseWordFromList chooses a word from LIST, determined by SEED
func chooseWordFromList(list []byte, seed []byte) []byte {
	seedScalar := 0
	for _, b := range seed {
		seedScalar += int(b) //TODO evaluate this
	}
	return getLine(list, seedScalar%len(list))
}
