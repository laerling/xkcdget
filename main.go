/*******************************************************************************
*
* Copyright 2019 laerling <laerling@posteo.de>
*
* This program is free software: you can redistribute it and/or modify it under
* the terms of the GNU General Public License as published by the Free Software
* Foundation, either version 3 of the License, or (at your option) any later
* version.
*
* This program is distributed in the hope that it will be useful, but WITHOUT
* ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
* FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
* details.
*
* You should have received a copy of the GNU General Public License along with
* this program. If not, see <http://www.gnu.org/licenses/>.
*
*******************************************************************************/

package main

import (
	"bufio"
	"bytes"
	"encoding/binary"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path"
	"strings"
	"syscall"

	"github.com/tilinna/z85"
	"golang.org/x/crypto/ssh/terminal"
)

func main() {
        version := "2.0.0"

	// print commit of build
	// write to stderr so that it doesn't get piped into xsel
	os.Stderr.Write([]byte("xkcdget " + version + "\n"))

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

	// ask for domain if not provided on command line
	args := os.Args[1:]
	domainArgProvided := false
	for _, arg := range args {
		if arg[0] != '-' {
			domainArgProvided = true
			break
		}
	}
	stdinReader := bufio.NewReader(os.Stdin)
	if !domainArgProvided {
		if terminal.IsTerminal(int(syscall.Stdin)) {
			os.Stderr.Write([]byte("Domain: "))
		}

		domain, err := stdinReader.ReadString('\n')
		// remove trailing newline
		domain = strings.Trim(domain, "\n")
		failOnError(err, "Cannot get domain from stdin")

		// prepend domain to arguments
		args = make([]string, 0, len(os.Args))
		args = append(args, domain)
		if len(os.Args) > 1 {
			args = append(args, os.Args[1:]...)
		}
	}

	// call pwget
	pwgetCmd := exec.Command(pwgetExe, args...)
	if terminal.IsTerminal(int(syscall.Stdin)) {
		pwgetCmd.Stdin = os.Stdin
	} else {
		pwgetCmd.Stdin = stdinReader
	}
	pwgetCmd.Stderr = os.Stderr
	key, err := pwgetCmd.Output()
	failOnError(err, "Running pwget failed")

	// exit if pwget was called for revocation
	for _, arg := range args {
		if arg == "-r" || arg == "--revoke" {
			os.Exit(0)
		}
	}

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
