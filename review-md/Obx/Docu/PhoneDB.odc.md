**Overview by Example: ObxPhoneDB**

This example is described in [<u>chapter 3</u>](../../Docu/Tut-3.odc.md) of the BlackBox tutorial.

DEFINITION ObxPhoneDB;

    TYPE String = ARRAY 32 OF CHAR;

    PROCEDURE LookupByIndex (index: INTEGER; OUT name, number: String);

    PROCEDURE LookupByName (name: String; OUT number: String);

    PROCEDURE LookupByNumber (number: String; OUT name: String);

END ObxPhoneDB.

Module ObxPhoneDB provides access to a phone database. Access may happen by index, by name, or by number. An entry consists of a name and a phone number string. Neither may be empty. The smallest index is 0, and all entries are contiguous.

PROCEDURE **LookupByIndex** (index: INTEGER; OUT name, number: ARRAY OF CHAR)

Return the <name, number> pair of entry *index*. If the index is too large, <"", ""> is returned.

The procedure operates in constant time.

Pre

index >= 0    20

Post

index is legal

    name # ""  &  number # ""

index is not legal

    name = ""  &  number = ""

PROCEDURE **LookupByName** (name: ARRAY OF CHAR; OUT number: ARRAY OF CHAR)

Returns a phone number associated with *name*, or "" if no entry for *name* is found.

The procedure operates in linear time, depending on the size of the database.

Post

name found

    number # ""

name not found

    number = ""

PROCEDURE **LookupByNumber** (number: ARRAY OF CHAR; OUT name: ARRAY OF CHAR)

Returns the name associated with *number*, or "" if no entry for *number* is found.

The procedure operates in linear time, depending on the size of the database.

Post

number found

    name # ""

number not found

    name = ""

