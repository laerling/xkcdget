# xkcdget
[pwget](https://github.com/majewsky/pwget) frontend for humans, implementing the [xkcd scheme](https://xkcd.com/936/).
This makes it possible to remember a password better and transcribe it from one screen to another more easily.

You can finally use pwget for login credentials too! Bonus: When you forget your login password you can retrieve it with an arbitrary machine that has pwget installed.
The only password you really need to remember from now on is the master password.

*Tip*: Remember a generated password like `creative_ball_see_author` like e. g. `the creative ball sees the author`. This pseudo-sentence makes a bit more sense than four arbitrary words in a row.

## The word lists

The word lists originate from [here (adjectives)](http://www.talkenglish.com/vocabulary/top-500-adjectives.aspx), [here (nouns)](http://www.talkenglish.com/vocabulary/top-1500-nouns.aspx), and [here (verbs)](http://www.talkenglish.com/vocabulary/top-1000-verbs.aspx).

## Disclaimer

The word lists are only so long. Their current use yields an entropy of `log(528*1525*1011*1525)/log(2) = 40.17514845593669 bit`
That is of course far less entropy than 256 bit, which is what pwget produces.

I'm writing this pwget frontend for my own use, and I personally prefer easy memorization to high entropy.
If you are in desperate need of entropy, fork this repo and replace the wordlists with your own (I recommend the [english aspell dictionary](http://ftp.gnu.org/gnu/aspell/dict/0index.html)) or increase the number of used words.
