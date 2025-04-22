# xkcdget
Deterministic password generator, implementing the [xkcd scheme](https://xkcd.com/936/).
That scheme makes it possible to remember a password better and transcribe it from one screen to another more easily.
xkcdget used to be based on [pwget](https://github.com/majewsky/pwget), but has been reimplemented as a standalone executable.

you can finally use a deterministic password generator for operating system login credentials too!
Bonus: When you forget your login password you can retrieve it with an arbitrary machine that has xkcdget installed.
With an appropriate terminal (e.&nbsp;g. [Termux](https://termux.com/) on Android) you can even use xkcdget on your phone to retrieve your passwords anywhere.
The only password you need to remember from now on is the master password.

Passwords consist of four words, concatenated camel-case style, suffixed by '\_1'. This makes most password prompts happy, since it contains upper-case, lower-case, digits, and special characters.
*Tip*: Remember a generated password like `CreativeBallSeeAuthor_1` by thinking of it as a sentence; `The creative ball sees the author.`
This pseudo-sentence makes a bit more sense than four arbitrary words in a row and can easily be imagined visually.

## The word list

The word list originates from [here (adjectives)](http://www.talkenglish.com/vocabulary/top-500-adjectives.aspx), [here (nouns)](http://www.talkenglish.com/vocabulary/top-1500-nouns.aspx), and [here (verbs)](http://www.talkenglish.com/vocabulary/top-1000-verbs.aspx).

## Disclaimer

The word lists are only so long. Their current use yields an entropy of `log(528*1525*1011*1525)/log(2) = 40.17514845593669 bit`
That is of course far less entropy than 256 bit, which is what pwget produces.

I'm writing this password generator frontend for my own use, and I personally prefer easy memorization to high entropy.
If you are in desperate need of entropy, fork this repo and replace the wordlists with your own (I recommend the [english aspell dictionary](http://ftp.gnu.org/gnu/aspell/dict/0index.html)) or increase the number of used words.
