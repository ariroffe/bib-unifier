# bib-unifier

Rust binary project for unifying a series of .bib files.

## Usage

```commandline
$ bib_unifier --help

bib_unifier 0.1.0
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

    -h, --help
            Print help information

    -V, --version
            Print version information
```

## Examples

###Simplest
```commandline
$ bib_unifier bib_files/test_files -s
Unifiying bibliography...
Found 3 repetitions in the bibliography.
Unified bibliography was written to "bib_files/test_files/[bib_unifier]bibliography.bib".
```

The program will look into any .bib files in the specified directory, read them, eliminate
the repetitions among them, and concatenate them into a single output file.

###Changing the output file

By default, the output file is named "[bib_unifier]bibliography.bib", and is placed in the same
directory as the one given as input. 

Note that the program is set to ignore files named
"[bib_unifier]bibliography.bib". That is so that, if you run the program again (with the same or
different parameters) does not take previously generated output as new input.

If you wish to change the output path, you can do so with the `-o` or `--output` flags:
```commandline
$ bib_unifier bib_files/test_files -s -o bib_files/test_files/output.bib
Unifiying bibliography...
Found 3 repetitions in the bibliography.
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
Entries:

1- @article{humberstone1996,
journal = {Journal of Philosophical Logic},
pages = {451--461},
author = {Lloyd Humberstone},
number = {5},
publisher = {Springer},
volume = {25},
ISSN = {00223611, 15730433},
year = {1996},
title = {Valuational Semantics of Rule Derivability},
}

2- @article{humberstone1996rep,
ISSN = {00223611, 15730433},
title = {Valuational Semantics of Rule Derivability},
pages = {451--461},
volume = {25},
year = {1996},
author = {Lloyd Humberstone},
publisher = {Springer},
number = {5},
journal = {Journal of Philosophical Logic},
}

are similar. Do you wish to keep the first (1), the second (2) or both (3)?
Enter your choice: 1

Entries:

1- @article{Prior1960,
volume = {21},
eprint = {https://academic.oup.com/analysis/article-pdf/21/2/38/360684/21-2-38.pdf},
year = {1960},
author = {Arthur N. Prior},
month = {12},
journal = {Analysis},
number = {2},
issn = {0003-2638},
doi = {10.1093/analys/21.2.38},
url = {https://doi.org/10.1093/analys/21.2.38},
pages = {38-39},
title = {{The Runabout Inference-Ticket}},
}

2- @article{Prior1960,
author = {Arthur Prior},
month = {11},
pages = {36-39},
title = {{The Runabout Inference-Ticket}},
issn = {0003-2639},
url = {https://doi.org/10.1093/analys/21.2.38},
eprint = {https://academic.oup.net/analysis/article-pdf/21/2/38/360684/21-2-38.pdf},
doi = {10.1093/analys/21.2.38},
volume = {20},
number = {3},
journal = {Analysis1},
year = {1961},
}

are similar. Do you wish to keep the first (1), the second (2) or both (3)?
Enter your choice: 3

Found 2 repetitions in the bibliography.
Unified bibliography was written to "bib_files/test_files/[bib_unifier]bibliography.bib".
```

Notice that the program states that it found 2 repetitions, even though we told it that the Prior articles are two different ones.
This is because, in the input files, there is another repeated entry that has all fields identical. That one
was removed without asking the user.

Also note that the citation keys of the two articles we decided to keep were identical. This might cause
problems for other software reading this file. Therefore, `bib_unifier` renames the second key to `Prior1960_1`.
If it found another file with the same key (either similar or not) it would save it as `Prior1960_2`, and so on.

Finally, you may see that the fields of the entries are printed in different order. This does not reflect a
difference in the input files (they are actually stored in the same order there). When they are both printed to
the console and written to the file, they will be ordered randomly. This is a behavior of a dependency crate,
I might consider fixing this later on.

### Using the similarity threshold

By default, the program compares the title and doi fields to know if two entries might be the same.
However, sometimes two entries that are actually the same have slightly different titles (and no doi set).
For these cases you may use the similarity threshold.

By default, it is set to `1`. But a number greater than zero and lower than one will make the program 
consider titles as possibly repeated even if they are not identical. For this, it implements various string similarity metrics.
To wit: 
- the normalized [Levenshtein edit distance](https://en.wikipedia.org/wiki/Levenshtein_distance) (default) 
- the normalized [Damerau-Levenshtein distance](https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance)
- The [Jaro and Jaro-Winkler](https://en.wikipedia.org/wiki/Jaro%E2%80%93Winkler_distance) distances
- The [Sørensen-Dice coefficient](https://en.wikipedia.org/wiki/S%C3%B8rensen%E2%80%93Dice_coefficient)

All range between `0` and `1`, where `1` is most similar and `0` least. If you are unsure about which metric to use, you should just leave the default option.

You can set the similarity threshold with the `-t` and `--threshold` flags (the default is `1`), and the
metric with the `a` and `--algorithm` flags (see the available options in the `--help` above).

```commandline
$ bib_unifier bib_files/test_files
Unifiying bibliography...

... [the same entries it found before, plus:]

Entries:

1- @incollection{BPS2018-WIAPL,
series = {Trends in Logic},
title = {{What is a Paraconsistent Logic?}},
publisher = {Springer},
booktitle = {{Between Consistency and Inconsistency}},
year = {2018},
editor = {Walter Carnielli and Jacek Malinowski},
author = {Barrio, Eduardo and Pailos, Federico and Szmuc, Damian},
pages = {89--108},
address = {Dordrecht},
}

2- @incollection{BPS2018-WIAPL,
booktitle = {{Between Consistency and Inconsistency}},
series = {Trends in Logic},
editor = {Walter Carnielli and Jacek Malinowski},
title = {{What is a paraconsistent logic?}},
publisher = {Springer},
pages = {89--108},
year = {2018},
author = {Barrio, Eduardo and Pailos, Federico and Szmuc, Damian},
address = {Dordrecht},
}

are similar. Do you wish to keep the first (1), the second (2) or both (3)?
Enter your choice: 2

Entries:

1- @book{Carnap1942,
title = {Introduction to Semantics},
series = {Studies in Semantics},
year = {1942},
publisher = {Harvard University Press},
author = {Rudolf Carnap},
}

2- @book{Carnap1942,
author = {Rudolf Carnap},
series = {Studies in Semantics},
title = {An Introduction to Semantics},
publisher = {Harvard University Press},
year = {1942},
}

are similar. Do you wish to keep the first (1), the second (2) or both (3)?
Enter your choice: 2

Found 5 repetitions in the bibliography.
Unified bibliography was written to "bib_files/test_files/[bib_unifier]bibliography.bib".
```

Note that the title comparison is case-sensitive (the BPS case was found with a similarity threshold of `0.7`
but not with `1`)

## Credits and License

Ariel Jonathan Roffé (CONICET, UBA)

This project is distributed under an MIT license (see the corresponding file).