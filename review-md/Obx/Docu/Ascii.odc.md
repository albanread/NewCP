**Overview by Example: ObxAscii**

Many times the input data to a program is given as an ASCII text file, or a given program specification dictates that the output format be plain ASCII text. This example shows how to process ASCII text files with the BlackBox Component Builder and presents a sketch of a module that provides a simple interface to handle ASCII text files.

The BlackBox Component Builder departs from the traditional I/O model found in other libraries. Module *Files* provides classes that abstract files, file directories, and access paths to open files, rather than cramming everything into one abstraction. So-called readers and writers represent positions in an open file. Several readers and writers may be operating on the same file simultaneously. The file itself represents the data carrier proper and may contain arbitrary data.

Textual information is handled by the text subsystem. Similar to the file abstraction, module *TextModels* provides a data carrier class - the text proper - and classes for reading characters from, and inserting characters into a text. Module *TextMappers* provides formatting routines to write values of the basic types of the Component Pascal language to texts. It also provides a scanner class that reads texts and converts the characters into integers, reals, strings, etc.

Texts may be stored in different formats on files. An ASCII text is just a special case of a text that contains no style information. So-called converters are used to handle different file formats. A converter translates the byte stream in a file into a text object in memory, and vice-versa.

The file and text abstractions are simpler, yet more flexible and powerful than the traditional I/O model. As with everything, this flexibility has its price. The simple, linear processing of a (text-)file requires some more programming to initialize the converter, text and formatter objects before a text file can be read.

This example demonstrates how to process ASCII text files with BlackBox. Module *ObxAscii* implements a simple, traditional interface for formatted textual input and output. It is by no means complete. *ObxAscii* may however serve as a model for implementing a more complete interface.

The implementation of data type *Text* is hidden. This renders possible a different implementation than the one presented below, e.g. using a traditional I/O library.

The field *done* of type *Text* indicates the success of the last operation. Procedure *Open* opens an existing file for reading. Procedure *NewText* creates a new, empty text for writing. Mixed reading and writing on the same text is not very common and is therefore not supported in this simple model. For new texts to become permanent, procedure *Register* must be used to enter the file in the directory. Finally, a set of *Write* procedures produce formatted output and the corresponding *Read* procedures read formatted data from texts.

A scanner and a formatter are associated with each text. In order to read ASCII text files and convert them to text objects, the converter for importing text files is needed. The initialization code in the module body finds the appropriate converter in the list of registered file converters and keeps a reference to it in the global variable *conv*.

A locator and a string is used to specify a directory and a file name when calling *Open* or *Register*. If the locator is *NIL*, the string given in parameter *name* is interpreted as a path name. (We use the well-established cross-platform URL-syntax for path names, i.e., directory names are separated by / characters.) The procedure *PathToFileSpec* produces a locator and file name from a path name.

Procedure *Open* uses *Converters.Import* with the ASCII text file converter stored in the global variable *conv* to initialize a text object with the contents of the file. The scanner is initialized and set to the beginning of the text. Procedure *NewText* just creates a new, empty text and initializes the formatter. Procedure *Register* uses *Converters.Export* with the ASCII text file converter to externalize a text to a file.

The *Read* procedures first check whether the text has been opened for reading and then use the scanner to read the next token from the text. If the token read by the scanner matches the desired type, the field *done* is set to *TRUE* to indicate success. The *Write* procedures first check whether the text has been opened for writing, i.e., created with *NewText*, and then use the formatter to write values to the text.

[<u>ObxAscii  sources</u>](../Mod/Ascii.odc.md)

