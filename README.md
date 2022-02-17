# bib-unifier

Rust binary project for unifying a series of .bib files.

*This project is still in early development and should not be used in production*

## Usage

```commandline
$ bib_unifier --help

bib_unifier 0.1.1
Ariel Jonathan Roffé <arielroffe@filo.uba.ar>
Unifies a set of .bib files into a single file, deleting repetitions

USAGE:
    bib_unifier [OPTIONS] <PATH>

ARGS:
    <PATH>    Directory where the .bib files are located

OPTIONS:
    -o, --output <PATH>
            Path (directory + filename) to the desired output file

    -s, --silent
            If present, will not ask for input regarding which repeated entry to keep

    -t, --threshold <SIMILARITY_THRESHOLD>
            Value between 0 and 1 to compare entry titles [default: 1]

    -a, --algorithm <ALGORITHM>
            Algorithm to use to compare similarity [default: levenshtein] [possible values:
            levenshtein, damerau-levenshtein, jaro, jaro-winkler, sorensen-dice]

    -b, --biblatex
            Default format for entries is bibtex. Setting this flag changes it to biblatex
    
    -h, --help
            Print help information

    -V, --version
            Print version information
```

## Examples

### Simplest usage
```commandline
$ bib_unifier bib_files/test_files -s
Unifiying bibliography...
Found 3 repetitions in the bibliography.
Unified bibliography was written to "bib_files/test_files/[bib_unifier]bibliography.bib".
```

The program will look into any .bib files in the specified directory, read them, eliminate
the repetitions among them, and concatenate them into a single output file.

### Changing the output file

By default, the output file is named "[bib_unifier]bibliography.bib", and is placed in the same
directory as the one given as input. 

Note that the program is set to ignore files that begin with
"[bib_unifier]". That is so that, if you run the program again (with the same or
different parameters) it does not take previously generated output as new input.

If you wish to change the output path, you can do so with the `-o` or `--output` flags:
```commandline
$ bib_unifier bib_files/test_files -s -o bib_files/test_files/output.bib
Unifiying bibliography...
Found 5 repetitions in the bibliography.
Unified bibliography was written to "bib_files/test_files/output.bib".
```

If the file specified as output already exists, it will overwrite it. Otherwise, it will create it.

### Choosing which files to keep

The above examples use the `-s` (silent) flag. If you remove it, when the program finds two 
repeated entries that differ in at least one field, it will ask which you want to keep. With the `-s` flag
the program always chooses the first variant it encounters.

```commandline
$ bib_unifier bib_files/test_files
Unifiying bibliography...
The following entries have the same title:

1- @article{humberstone1996,
author = {Lloyd Humberstone},
ISSN = {00223611, 15730433},
journal = {Journal of Philosophical Logic},
number = {5},
pages = {451--461},
publisher = {Springer},
title = {Valuational Semantics of Rule Derivability},
volume = {25},
year = {1996},
}

2- @article{humberstone1996rep,
author = {Lloyd Humberstone},
ISSN = {00223611, 15730433},
journal = {Journal of Philosophical Logic},
number = {5},
pages = {451--461},
publisher = {Springer},
title = {Valuational Semantics of Rule Derivability},
volume = {25},
year = {1996},
}

Do you wish to keep the first (1), the second (2) or both (3)?
Enter your choice: 

[...]

Found 5 repetitions in the bibliography.
Unified bibliography was written to "bib_files/test_files/[bib_unifier]bibliography.bib".
```

For any repeated entries it finds, it will ask which you want to keep only if they are not identical in
key & all fields. If they are, it will not ask and keep just one copy.

Repeated entries are detected as those that have:

- The same key (in this case, keeping both will make it rename the second key to "originalkey(1)", and so on)
- The same doi (if present)
- The same title
- Similar title (see below)

The program will check in that order.


### Using the similarity threshold

By default, when looking at entry titles, the program compares if they are identical too see if they might be the same.
However, sometimes two entries that are actually the same have slightly different titles.
For these cases you may use the similarity threshold.

By default, it is set to `1`. But a number greater than zero and lower than one will make the program 
consider titles as possibly repeated even if they are not identical. For this, it implements various string similarity metrics.
To wit: 
- the normalized [Levenshtein edit distance](https://en.wikipedia.org/wiki/Levenshtein_distance) (default) 
- the normalized [Damerau-Levenshtein distance](https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance)
- The [Jaro and Jaro-Winkler](https://en.wikipedia.org/wiki/Jaro%E2%80%93Winkler_distance) distances
- The [Sørensen-Dice coefficient](https://en.wikipedia.org/wiki/S%C3%B8rensen%E2%80%93Dice_coefficient)

All range between `0` and `1`, where `1` is most similar and `0` least. 
If you are unsure about which metric to use, you should just leave the default option.

You can set the similarity threshold with the `-t` and `--threshold` flags, and the
metric with the `a` and `--algorithm` flags (see the available options in the `--help` above).

```commandline
$ bib_unifier bib_files/test_files -t 0.7
Unifiying bibliography...

[...]

The following entries have the similar titles:

1- @incollection{BPS2018-WIAPL_1,
address = {Dordrecht},
author = {Barrio, Eduardo and Pailos, Federico and Szmuc, Damian},
booktitle = {{Between Consistency and Inconsistency}},
editor = {Walter Carnielli and Jacek Malinowski},
pages = {89--108},
publisher = {Springer},
title = {{What is a paraconsistent logic?}},
series = {Trends in Logic},
year = {2018},
}

2- @incollection{BPS2018-WIAPL,
address = {Dordrecht},
author = {Barrio, Eduardo and Pailos, Federico and Szmuc, Damian},
booktitle = {{Between Consistency and Inconsistency}},
editor = {Walter Carnielli and Jacek Malinowski},
pages = {89--108},
publisher = {Springer},
series = {Trends in Logic},
title = {{What is a Paraconsistent Logic?}},
year = {2018},
}

Do you wish to keep the first (1), the second (2) or both (3)?
Enter your choice: 1

The following entries have the similar titles:

1- @book{Carnap1942_1,
author = {Rudolf Carnap},
publisher = {Harvard University Press},
series = {Studies in Semantics},
title = {An Introduction to Semantics},
year = {1942},
}

2- @book{Carnap1942,
author = {Rudolf Carnap},
publisher = {Harvard University Press},
series = {Studies in Semantics},
title = {Introduction to Semantics},
year = {1942},
}

[...]

Found 7 repetitions in the bibliography.
Unified bibliography was written to "bib_files/test_files/[bib_unifier]bibliography.bib".
```

Note that the title comparison is case-sensitive (the BPS case is found with a similarity threshold of `0.7`
but not with `1`)


### Bibtex vs biblatex format

If you include the `-b` or `--bibtex` flag, entries will be printed and saved with a slightly different format. e.g.:

```commandline
$ bib_unifier bib_files/test_files -b
Unifiying bibliography...
The following entries have the same title:

1- @article{humberstone1996,
author = {Lloyd Humberstone},
ISSN = {00223611, 15730433},
journaltitle = {Journal of Philosophical Logic},
number = {5},
pages = {451--461},
publisher = {Springer},
title = {Valuational Semantics of Rule Derivability},
volume = {25},
year = {1996},
}

[...]

```

Note that it uses 'journaltitle' instead of 'journal'. There are other slight differences in format, run with both
options see which you like best.


## Credits and License

Ariel Jonathan Roffé (CONICET, UBA)

This project is distributed under an MIT license (see the corresponding file).