**SqlDB**

DEFINITION SqlDB;

    CONST

        names = -1;

        converted = 1; truncated = 2; overflow = 3; incompatible = 4; noData = 5;

        sync = FALSE; async = TRUE;

        hideErrors = FALSE; showErrors = TRUE;

    TYPE

        String = POINTER TO ARRAY OF CHAR;

        Row = RECORD

            fields: POINTER TO ARRAY OF String

        END;

        Blob = RECORD

            len: INTEGER;

            data: POINTER TO ARRAY OF BYTE

        END;

        Command = PROCEDURE (par: ANYPTR);

        TableCommand = PROCEDURE (t: Table; par: ANYPTR);

        Database = POINTER TO ABSTRACT RECORD

            res: INTEGER;

            async: BOOLEAN;

            showErrors: BOOLEAN;

            (d: Database) Exec (statement: ARRAY OF CHAR), NEW, ABSTRACT;

            (d: Database) NewTable (): Table, NEW, ABSTRACT;

            (d: Database) Call (command: Command; par: ANYPTR), NEW, ABSTRACT;

            (d: Database) Commit, NEW, ABSTRACT;

            (d: Database) Rollback, NEW, ABSTRACT

        END;

        Table = POINTER TO ABSTRACT RECORD

            base-: Database;

            rows, columns: INTEGER;

            res: INTEGER;

            strictNotify: BOOLEAN;

            (t: Table) InitBase (base: Database), NEW;

            (t: Table) Exec (statement: ARRAY OF CHAR), NEW, ABSTRACT;

            (t: Table) Available (): BOOLEAN, NEW, ABSTRACT;

            (t: Table) Read (row: INTEGER; VAR data: ANYREC), NEW, ABSTRACT;

            (t: Table) Clear, NEW, ABSTRACT;

            (t: Table) Call (command: TableCommand; par: ANYPTR), NEW, ABSTRACT

        END;

    VAR debug: BOOLEAN;

    PROCEDURE  OpenDatabase (protocol, id, password, datasource: ARRAY OF CHAR;

                                                    async, showErr: BOOLEAN; OUT d: Database; OUT res: INTEGER);

END SqlDB.

Module *SqlDB* is the application programming interface for applications using relational databases supporting the SQL language.

**names**

The constant *names* can be used as the *row* parameter in a table's* Read* method. This is legal if the passed interactor is of type *Row. *The interactor is filled with the column names of the actual table.

**converted**

Possible value of a table's *res* field after a *Read* operation. Indicates that some database values had to be converted to match the type of the corresponding interactor field.

**truncated**

Possible value of a table's *res* field after a *Read* operation. Indicates that some string values had to be truncated to fit in the corresponding interactor field.

**overflow**

Possible value of a table's *res* field after a *Read* operation. Indicates that some numeric values could not be read because they where too large to fit in the corresponding interactor field.

**incompatible**

Possible value of a table's *res* field after a *Read* operation. Indicates that some database values could not be read because of a type incompatibility

**noData**

Possible value of a table's *res* field after a *Read* operation. Indicates that the *row* parameter pointed to a non-existing row.

**sync**

Possible value for the *async* parameter of *OpenDatabase*.

**async**

Possible value for the *async* parameter of *OpenDatabase*.

**hideErrors**

Possible value for the *showErr* parameter of *OpenDatabase*, and of the *showErrors* field of a database object.

**showErrors**

Possible value for the *showErr* parameter of *OpenDatabase*, and of the *showErrors* field of a database object.

TYPE **Row**

A *Row* variable contains a textual representation of a single row of a database. An interactor of this type can be used as parameter of the *Table.Read* operation when a textual representation of an unknown table is to be retrieved.

**fields**: POINTER TO ARRAY OF String

The actual fields of the table row. The array is allocated by the *Table.Read* operation. *fields* is allocated by *SqlDB* if it is *NIL* or the length is not correct, otherwise the existing array is reused. The same holds for the string pointers in *fields*, except that a string is not reallocated if the new value fits into the old string.

TYPE **Blob**

Used for the representation of data of any kind and size ("binary large object").

**len**: INTEGER    len >= 0

The length of the data in bytes.

**data**: POINTER TO ARRAY OF BYTE     len > 0 => data # NIL

The binary data stored as an array of character. There is no special termination symbol in the array.

TYPE **Par**

Basetype for extensible parameter lists.

TYPE **Command**

Procedure type with extensible parameter list, used in *Database.Call.*

TYPE **TableCommand**

Procedure type with table argument and extensible parameter list, used in *Table.Call*.

TYPE **Database**

ABSTRACT

A *Database* variable represents an SQL database.

**res**: INTEGER

Result value; valid after opening the database, and after each call to *Database.Exec* or *Database.NewTable*.

**async**: BOOLEAN

Set from the *async* parameter of *OpenDatabase*. *async = TRUE* means asynchronous operation is allowed.

**showErrors**: BOOLEAN

Can be set to enforce the database driver to display verbose error messages if an SQL statement is incorrect.

PROCEDURE (d: Database) **Exec** (statement: ARRAY OF CHAR)

NEW, ABSTRACT

Execute the SQL statement passed as parameter. It must not be a table-returning statement.

If *d* has been opened in *async* mode, the evaluation of *statement* is only started; it will be completed asynchronously.

The following portable error codes may be returned in *res*:

    5    outOfTables

    6    notExecutable

    9    tooManyBlobs

Warning: don't use the SQL transaction statements (commit / abort), since they may interfere with *SqlDB*. Instead, call *db.Commit* or *db.Abort*.

Pre

statement # ""    20

statement is not table-returning    21    (* may be delayed *)

Post

res denotes error value (0 if no error occurred)

PROCEDURE (d: Database) **NewTable** (): Table

NEW, ABSTRACT

Allocate a new table object. A table represents the result table of the most recently executed SQL statement on it.

Post

result # NIL

    d.res = 0

    result.base = d

    result.rows = 0

    result.columns = 0

    result.res = 0

    result.strictNotify = FALSE

result = NIL

    d.res # 0

PROCEDURE (d: Database) **Call** (command: Command; par: Par);

NEW, ABSTRACT

The procedure *command(par)* is executed. *par* may be *NIL*. Inside *command* no table based operations (*Table.Read*, *Table.Exec*, *Table.Clear*, or *Table.Call*) may be invoked on an existing table. Operations on tables allocated (with *Database.NewTable*) inside the command are legal.

This procedure is useful in asynchronously executing statements.

Pre

command # NIL    20

PROCEDURE (d: Database) **Commit**

NEW, ABSTRACT

Commits the currently executing transaction. Note that a transaction need not be started explicitly.

Do *not* use the database's SQL transaction commands directly.

PROCEDURE (d: Database) **Rollback**

NEW, ABSTRACT

Aborts the currently executing transaction. Note that a transaction need not be started explicitly.

Do *not* use the database's SQL transaction commands directly.

TYPE **Table**

ABSTRACT

A table represents the result of an SQL query, typically of a SELECT statement. The result table is a snapshot, i.e., retains the state which was valid when the query has been performed.

**base**-: Database    base # NIL

The database to which the table belongs.

**rows**: INTEGER    rows >= 0

The number of rows that the most recent query returned. If the actual database driver cannot return the number of rows in a query, rows is set to *MAX(INTEGER)*.

**columns**: INTEGER    columns >= 0

The number of columns that the most recent query returned.

*columns = 0* means no table was returned.

**res**: INTEGER

The result value of the most recent query (*0* if no error).

**strictNotify**: BOOLEAN

After one or several SQL queries have been performed on the table and BlackBox is idle again, a *Dialog.Update* is performed automatically by an action (-> Services). This is normally the desired behavior. If the result state *after each query* should be reflected in controls bound to the table's interactor, *strictNotify* should be set. This may be desired for long-running commands, so that the user sees progress while the command is running.

PROCEDURE (t: Table) **InitBase** (base: Database)

NEW

Initialize the table's *base* pointer.

*InitBase* is called internally.

Pre

base # NIL    20

t.base = NIL  OR  t.base = base    21

Post

t.base = base

PROCEDURE (t: Table) **Exec** (statement: ARRAY OF CHAR)

NEW, ABSTRACT

Execute the SQL statement passed as parameter. It must be a row-returning statement. *Exec* first performs *t.Clear* in order to flush the old result table, then executes the query.

If *t.base* has been opened in *async* mode, the evaluation of *statement* is only started and will be completed asynchronously.

The following portable error codes may be returned in *res*:

    5    outOfTables

    6    notExecutable

    9    tooManyBlobs

Pre

statement # ""    20

t is legal table in enclosing command    21

Post

res denotes error value (0 if no error occurred).

PROCEDURE (t: Table) **Available** (): BOOLEAN;

NEW, ABSTRACT

Tells whether *Read* can execute immediately. This procedure is rarely used. It should be used before a *Read* is performed, if it is not guaranteed that the previous *Exec* has already returned a result. This may only happen if *Exec* is performed asynchronously. Note that *Available()* only guarantees that a result table has become available, but this table may well be empty (*columns* = 0 and *rows* = 0).

PROCEDURE (t: Table) **Read** (row: INTEGER; VAR data: ANYREC)

NEW, ABSTRACT

Reads the *row'th* row of the result table into the interactor *data*.

The generic data type *Row* which consists of an open array of strings can be used to read a row of data from an arbitrary table. All table values are converted to a string representation in this case. Otherwise the fields of the data record must be of one of the following types:

BYTE, SHORTINT, INTEGER, SHORTREAL, REAL, BOOLEAN, ARRAY n OF CHAR; POINTER TO ARRAY OF CHAR, Dialog.Currency, Dialog.List, Dialog.Combo, Dates.Date, Dates.Time, or SqlDB.Blob. Other array and record types are handled recursively. Pointers are dereferenced, and the bound structures are handled recursively. Pointers must not be NIL with the exception of POINTER TO ARRAY OF CHAR. If such a pointer is NIL or the bound array is too small to receive the corresponding string, a new array of suitable length is allocated automatically.

Four levels of complications can arise during the read:

ꀢ A value has to be converted from one data type to another. Conversions from any type to strings (ARRAY n OF CHAR or POINTER TO ARRAY OF CHAR) are always supported. Other conversions are driver-dependent.

ꀢ A string need to be truncated because the destination array is too small.

ꀢ A numeric value is too large to be stored in the destination (overflow).

ꀢ A value cannot be assigned because its type is incompatible with the corresponding field. The field is cleared in such a situation.

After the call, *t.res* contains the most serious event (in the order *converted*, *truncated*, *overflow, incompatible*) that happened during the reading of all columns.

The column names of the table can be read into a variable of type *Row*, using the special value *names* for the *row* parameter.

It is permissible to read past the end of the table (*row* >= *t.rows*). The whole interactor is cleared in this case and *t.res* is set to *noData*.

Pre

t.columns > 0    20;    (* may be delayed *)

row >= 0  OR  row = names & data IS Row    21

data contains legal data types    22    (* may be delayed *)

data contains no NIL pointers    23    (* may be delayed *)

t.base.async => data is global variable    24

t is legal table in enclosing command    25

Post

t.res IN {0, converted, truncated, overflow, incompatible, noData}

PROCEDURE (t: Table) **Clear**

NEW, ABSTRACT

Releases the resources needed to represent the most recently created result table of *t*.

Pre

t is legal table in enclosing command    20

Post

t.rows = 0

t.columns = 0

t.res = 0

PROCEDURE (t: Table) **Call** (command: TableCommand; par: Par)

NEW, ABSTRACT

The procedure *command(t, par)* is executed. *par* may be *NIL*. Inside *command* no table based operations (*Table.Read*, *Table.Exec*, *Table.Clear*, or *Table.Call*) may be invoked on a preexisting table different from *t*. Operations on tables allocated (with *Database.NewTable*) inside the command are legal.

This procedure is useful for asynchronously executing commands.

Pre

command # NIL    20

t is legal table in enclosing command    21

VAR **debug**: BOOLEAN

Used internally. (Shows a protocol of what is happening during the execution of *SqlDB* commands.)

PROCEDURE  **OpenDatabase** (protocol, id, password, datasource: ARRAY OF CHAR;

                                                    async, showErr: BOOLEAN; OUT d: Database; OUT res: INTEGER)

Open the database specified by the parameters. *protocol* is the name of the module which contains the database driver, e.g., "SqlOdbc" or "SqlOdbc3". Note that these names are case sensitive. *id* is a driver-specific string which denotes the connection information for the network in use, or similar protocol-specific information. *password* is the password used to log into the DBMS. It may be empty; in this case the user may be prompted for the password, depending on the driver. *datasource* specifies the database to be opened. It should be a unique name for the particular protocol.

If the database described by *protocol* and *datasource* is already open in the same BlackBox environment, the same database connection may be used, without considering the password again.

Consult the driver documentation for further information on the parameters to be passed.

If *async* is true, the returned database object is allowed to behave asynchronously.

If *showErr* is true, the returned database displays verbose error messages when incorrect SQL statements are detected.

The following portable error codes may be returned in *res*:

    1    noDriverMod

    2    noDriverProc

    3    wrongDriverType

    4    connectionsExceeded

    7    cannotOpenDB

    8    wrongIdentification

Pre

protocol # ""    20

Post

res = 0

    database was correctly opened

    d # NIL

    d.async = async

    d.showErrors = showErr

res # 0

    database couldn't be opened

