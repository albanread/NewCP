**Sql Subsystem (only available for Windows)**

**Developer Manual**

**Contents**

[<u>Introduction</u>](#Introduction)

[<u>Overview</u>](#Overview)

[<u>Databases, tables, and interactors</u>](#Databases)

[<u>Interpreted embedded SQL</u>](#Dynamic)

[<u>Type Row</u>](#Type)

[<u>SqlBrowser</u>](#SqlBrowser)

[<u>Blobs</u>](#Blobs)

[<u>Asynchronous operation</u>](#Asynchronous)

[<u>SqlDrivers</u>](#SqlDrivers)

[<u>ODBC driver</u>](#ODBC)

[<u>SqlDB</u>](#SqlDB)

[<u>Clearing Database and Table pointers</u>](#Clearing)

[<u>Displaying tables</u>](#Displaying)

[<u>Design rules</u>](#Design)

[<u>Example</u>](#Example)

<a id="Introduction"></a>**Introduction**

One of the most important applications of personal computers is as database frontends, i.e., as user interfaces to a database management system (DBMS). For small applications, the database resides on the same computer as the frontend, thus only one single user can access the database at any one time. For more demanding applications, the database is moved to a central server, to which several client PCs are connected, e.g. via a local area network. A client provides a graphical user interface and runs the application software; the server performs data storage. Typically, several clients may access the same database concurrently.

Today, the most important DBMS systems are based on a relational data model and use the SQL language (Structured Query Language) as the means to define, retrieve, and manipulate data.

The Sql subsystem for BlackBox provides a simple, portable, extensible, and integrated access mechanism to SQL databases. It is extensible, i.e., different database implementations can be interfaced through suitable driver modules. Today, there exists one driver module: a generic one that uses Microsoft's ODBC (Open Database Connectivity) standard. The ODBC driver comes standard with BlackBox (*SqlOdbc* and *SqlOdbc3*, respectively).

It is assumed that the reader knows about database design, SQL, and BlackBox; in particular the latter's *Form* subsystem, controls, and module *Dialog*.

<a id="Overview"></a>**Overview**

The following goals have been pursued in the design of the Sql subsystem:

*Simplicity*

Existing interfaces to SQL are often very complicated to use. Sql provides an interface that is easier to learn and to use than other interfaces with a similar level of flexibility. The central programming interface (module *SqlDB*) only exports two major and a few minor data types. The major datatypes are the interface types *SqlDB.Database* and *SqlDB.Table*. They provide object-oriented abstractions for SQL databases and SQL result tables, respectively. Elements of a result table can be read into Component Pascal records, i.e., into objects that represent one single row of a table.

*Full access to SQL*

It has not been attempted to hide SQL. SQL is too powerful to be completely hidden behind "magic" database-aware controls, for example. Sql is oriented towards the professional developer, it is not attempted to provide "end-user" tools.

*Integration*

Often, SQL statements need to be constructed at run-time. In particular, actual parameters must be spliced into statements which contain symbolic place holders as formal parameters. In Sql, this need not be done manually by the programmer. Instead, BlackBox's built-in metaprogramming facilities are used to do it automatically. In fact, a dynamically interpreted kind of "embedded" SQL is realized, where Component Pascal variable names can be used directly in SQL statements.

*Extensibility*

Application programs use the *SqlDB* programming interface. This interface is implemented on top of a lower-level driver interface (*SqlDrivers*). This driver interface is defined in a component-oriented style, i.e., different driver implementations can be used without invalidating application programs. In fact, several different drivers could be used in the same application concurrently.

Figure a  Simple application interface, extensible driver interface

*Separation of program logic and user interface*

For clarity, maintainability, and maximum platform independence, user interface and program logic (the latter including the so-called "business logic") are separated as far as possible. A graphical user interface is treated as a merely cosmetic addition to the program, and can be modified without touching the core application's source code. In fact, a command button or a text entry field need not even be "database-aware" in order to invoke database operations or to represent database contents.

A program produces and consumes database contents via so-called interactor objects, i.e., Component Pascal records. Such record variables are used as sources and destinations of command parameters or result data.

Figure b  Interactors as switchboards for application program, DBMS, and user interface

*Framework-friendly, user-friendly*

A framework depends on the cooperation of the objects that it is composed of. An object cannot be considered cooperative if it blocks the others or the user for a long time. You shouldn't have to wait several minutes until you can continue with your work, just because a remote server machine takes so long to complete a database transaction.

For this reason, *SqlDB* supports a non-blocking asynchronous programming style in addition to the traditional blocking style. The programmer can decide which style is more appropriate for the given application. Non-blocking operation is especially important if an application may access a server database. Server queries sometimes take much longer than local queries. Blocking operations may be appropriate for simple local queries, or for batch processes that require no user interaction.

The non-blocking programming style is more involved than the straight-forward blocking style. It uses object-oriented programming principles to avoid an even more intricate and hard-to-debug programming style based on so-called threads (i.e., preemptive multitasking).

Even an application which uses a blocking programming style is non-modal, meaning that several data query or entry masks (windows) can be open at the same time. This is in the general spirit of BlackBox, which avoids modal dialog boxes wherever feasible. Ultimately it is the user who benefits, by not being locked into a single task at any time.

For an overview over the files of *Sql*, see its [<u>Map</u>](Sys-Map.odc.md) text.

<a id="Databases"></a>**Databases, tables, and interactors**

From a program's perspective, a database is represented by a *SqlDB.Database* object. As long as a database object is available, SQL commands can be executed. Execution may be local, or remote on a server.

The following declaration describes the *SqlDB.Database* type:

    Database = POINTER TO ABSTRACT RECORD

        res: INTEGER;

        async: BOOLEAN;

        showErrors: BOOLEAN;

        (d: Database) Exec (statement: ARRAY OF CHAR), NEW, ABSTRACT;

        (d: Database) Commit, NEW, ABSTRACT;

        (d: Database) Abort, NEW, ABSTRACT;

        (d: Database) Call (command: Command; par: ANYPTR), NEW, ABSTRACT;

        (d: Database) NewTable (): Table, NEW, ABSTRACT

    END;

Procedure *Exec* is used to execute SQL statements which don't return result tables; e.g., DELETE or INSERT statements, such as:

database.Exec("DELETE FROM Companies WHERE id = 5" )

Transactions are started automatically when the first modifying SQL command is executed. A transaction is terminated either by calling *Commit* or *Abort*.

Warning: don't use the database's SQL transaction statements, since they may interfere with *Commit* and *Abort*.

A database object is obtained by calling *SqlDB.OpenDatabase*:

PROCEDURE OpenDatabase (protocol, id, password,

                                                                datasource: ARRAY OF CHAR;

                                                                async: BOOLEAN;

                                                                OUT d: SqlDB.Database;

                                                                OUT res: INTEGER);

This procedure opens the database given by the pair (*protocol, datasource*). If that database is already open from a previous call to *OpenDatabase*, the same database connection may be used, without considering the *id* and *password* information again. The exact interpretation of the latter parameters depends on the database driver, whose name is given in *protocol* (e.g. *SqlOdbc3*).

If an application needs to fetch result tables from a database, typically generated by SELECT statements, it needs to provide table objects, which represent the returned result tables. The contents of a table is static, i.e., it represents a snapshot of the database contents at the time the statement is executed. Conceptually, a table is a local and independent copy of the database contents. Several tables can be used simultaneously.

A table object is obtained by calling its database object's *NewTable* procedure.

An *SqlDB.Table* is declared as

    Table = POINTER TO ABSTRACT RECORD

        base-: Database;

        rows, columns, res: INTEGER;

        strictNotify: BOOLEAN;

        (t: Table) Exec (statement: ARRAY OF CHAR), NEW, ABSTRACT;

        (t: Table) Available (): BOOLEAN, NEW, ABSTRACT;

        (t: Table) Read (row: INTEGER; VAR data: ANYREC), NEW, ABSTRACT;

        (t: Table) Clear, NEW, ABSTRACT;

        (t: Table) Call (command: TableCommand; par: ANYPTR), NEW, ABSTRACT

    END;

The pair *(rows, columns)* denotes the number of rows and columns of the most recently computed result table. A result table is generated by calling the table's *Exec* procedure. It may only be used with row-returning SQL statements, i.e., statements which return a (possibly empty) table. Typically, this is a SELECT statement such as

table.Exec("SELECT * FROM Companies WHERE id = 17)

*Read* can be used to read a row from the result table into an interactor (i.e. an exported record variable). The interactor can then be manipulated by the program or by its graphical user interface elements. For example, the following statement

table.Read(17, company);

reads the contents of row 17 of *table* into the variable *company*, which might be declared as

VAR

    company: RECORD

        id: INTEGER;

        name, ceo: ARRAY 32 OF CHAR;

        employees: INTEGER

    END;

Note that the field *table.rows* cannot be computed by all database drivers, some may return MAX(INTEGER) instead. To loop over all rows of a result table, it is therefore better to avoid using *table.rows* in the loop termination condition. Instead, looping can be done while *table.res # SqlDB.noData* holds.

Record fields and rows of a result table are matched in the order that they are defined in the record or in the database, respectively (SQL doesn't define an order, but every actual database product does). Record fields to be matched must be exported. If there are non-exported record fields, they are simply ignored.

The following Component Pascal types are interpreted by *SqlDB*:

    BOOLEAN

    BYTE, SHORTINT, INTEGER

    SHORTREAL, REAL

    ARRAY OF CHAR

    Dates.Date

    Dates.Time

    Dialog.Currency

    Dialog.List

    Dialog.Combo

    SqlDB.Blob

How these types are mapped to SQL data types depends on the actual SQL database product and the Sql driver. The following example table applies to Microsoft's Sql Server product which is accessed via ODBC:

**SQL    Component Pascal**

{bit, tinyint, smallint, integer, bigint}    {BOOLEAN(1), BYTE, SHORTINT, INTEGER, Dialog.List}

{real, float(p), double precision}    {SHORTREAL, REAL}

{char(n)(2), varchar(n)(3), long varchar}    {ARRAY OF CHAR, Dialog.Combo}

{decimal(p, s), numeric(p, s)}    Dialog.Currency

{date, timestamp(4)}    Dates.Date

{time, timestamp(4)}    Dates.Time

{binary(n), varbinary(n), long varbinay)    SqlDB.Blob

(1)  0 = FALSE, 1 = TRUE

(2)  character string of fixed string-length n

(3)  variable-length character string with a maximum string length n

(4) only the date or the time part is used, not both simultaneously

Note:

Values of any SQL data type can be read in a textual form, into an *ARRAY OF CHAR*.

SQL datetime values can be mapped either to *Dates.Date* or to *Dates.Time*, but not to both simultaneously.

<a id="Dynamic"></a>**Interpreted embedded SQL**

Consider the following simple SQL statement:

  SELECT * FROM Companies WHERE id = 249

It is easy to provide a programming interface which can execute this query, e.g., something like

  table.Exec("SELECT * FROM Companies WHERE id = 249")

However, things get messy when the SQL statement should be parameterized with program variables, e.g.,

  SELECT * FROM Companies WHERE id = searchId

where *searchId* is a global variable. Note that it would not be correct to call

  table.Exec("SELECT * FROM Companies WHERE id = searchId")

since the DBMS has no idea that the application program has a variable called *searchId*.

What could be done is something like the following:

  ConvertIntegerToString(searchId, str);

  str := "SELECT * FROM Companies WHERE id = " + str;

  table.Exec(str);

Some systems provide precompilers for so-called *embedded SQL*, where true statements of a programming language can be mixed with SQL statements, and these SQL statements may contain formal parameters. Syntactically, such formal parameters are preceeded by colons. In embedded SQL, the above example would look like this:

  SELECT * FROM Companies WHERE id = :searchId

Embedded SQL has drawbacks in that the precompiler typically produces code for one particular combination of programming language/compiler and DBMS only. Precompilers are often static in that they prevent the dynamic composition of SQL strings. They slow down compilation because of the preprocessing they have to perform.

BlackBox uses a novel approach which combines the convenience of embedded SQL with the flexibility of explicit SQL programming interfaces ("call level interfaces"). For this purpose, it uses its metaprogramming facilities (i.e., run-time type information) to access global variables, as they occur in an SQL string.

For example, if there is a global and exported integer variable *searchId* in module *Sample*, the following SQL statement can be used:

  "SELECT * FROM Companies WHERE id = :Sample.searchId"

*SqlDB* provides a procedure to execute such a string (*SqlDB.Database.Exec, SqlDB.Table.Exec*). These procedures replace all placeholders starting with a colon by the appropriate run-time values.

For additional convenience, whole record variables can be used in SQL strings. Their fields will be expanded suitably. For example, if there is a global and exported interactor *company* in module *Sample* with the fields *id*, *name*, *ceo*, *employees* then *SqlDB* will expand the following string

  INSERT INTO Companies VALUES (:Sample.company)

into

  INSERT INTO Companies VALUES (5, 'Macrosoft Corp.', 'Doors', 10000)

assuming that *company.id = 5, company.name = "Macrosoft Corp."*, *company.ceo *= "Doors" and *company.employees = *10000.

Note: A colon does not trigger string substitution if a Windows path name follows the colon (this is useful e.g. when using MS Access), as in "SELECT * FROM C:\directory\databaseName.tableName".

<a id="Type"></a>**Type Row**

Normally, a table's *Read* procedure is performed on an interactor that contains one record field per result table column. This is convenient, since it provides automatic mapping between relational data and Component Pascal objects, i.e., an object-oriented front-end for SQL databases.

However, specialized tools such as database browsers don't know about the exact definition of the tables they will be accessing, and thus cannot define the necessary interactor types in advance. For those special cases, a more general dynamic mechanism is provided.

Instead of passing a normal interactor to *Read*, a variable of the special type *SqlDB.Row* is passed for this purpose. As a result, it will contain an array of pointers to strings. Each string contains the textual representation of one table column of the row read. If *SqlDB.names* is passed as *row* parameter, the strings will contain the names of the table columns, instead of values.

    String = POINTER TO ARRAY OF CHAR;

    Row = RECORD

        fields: POINTER TO ARRAY OF String

    END

<a id="SqlBrowser"></a>**SqlBrowser**

Module *SqlBrowser* implements a database browsing utility. The user can interactively enter the protocol, id, password, and datasource parameters in a dialog box. A further text entry field allows to enter arbitrary SQL statements. Clicking on the "Execute" button or pressing Enter executes the query. If the database is not already open, it is opened automatically and remains open until the Browser dialog box is closed. If the statement returns a result table, it is displayed in its own window. For statements not returning a result, the message "statement executed" is displayed. In case of an error, a suitable error message appears.

The source of [<u>SqlBrowser</u>](../Mod/Browser.odc.md) is available. It demonstrates the use of *SqlDB.Row* to obtain data from tables that are not known at compile time, i.e., "dynamically".

<a id="Blobs"></a>**Blobs**

Binary Large Objects, or "blobs", allow to store unstructured data as large ARRAY OF BYTE variables. For example, large image bitmaps, serialized stores, or any other data may be stored in blobs. A blob is represented as a record

    Blob = RECORD

        len: INTEGER;

        data: POINTER TO ARRAY OF BYTE

    END

The field *len* indicates the number of valid bytes in *data*. Data is a pointer to the byte array. A *Blob* that is used in several *Read* operations reuses the same *data* array if possible, i.e. if the new data takes at most as many bytes as the previous result's data.

<a id="Asynchronous"></a>**Asynchronous operation**

There are different ways how a database can be accessed. In particular, access may occur to a local or to a remote server database. Access to remote databases may take considerably longer than access to local databases. If an operation takes long to complete, a user should not be blocked from other work during this time. This requires asynchronous operation, i.e., a query is started, but the user can immediately continue to work while the query is being completed.

Asynchronous operation is usually associated with preemptive multitasking, so-called *threads*. However, threads incur considerable additional complexity, often also for programmers who don't even use this feature. *Sql* uses a more light-weight approach to the problem. It allows to open a database in an asynchronous mode, such that any *Exec* statement only starts a query. The procedure immediately returns; i.e., it doesn't block the rest of the framework, and thus the user, from further work. When the result of the query is ready, the program is notified and can continue its work, using the results of the query.

Conceptually, the procedures *Exec, Clear, Read* and *Call* for an asynchronously accessed database are not executed right away when the procedure is called. Instead, the operations are queued in *SqlDB*, with one queue for the database and one queue for each table on this database. These queues are processed in the background. The results can be accessed when they are ready (indicated by *Table.Available*). Polling a table whether a pending *Exec* has returned a result may be done with a *Services.Action*. However, *SqlDB* provides a more convenient means, namely a table's or a database's *Call* procedure. The idea is that one or several asynchronous operations are started, and then a *continuation procedure* is set up by calling *Call(ContinuationProc, someData)*. When the previous operations in the queue have terminated, the continuation procedure is called. In the continuation procedure, it can be assumed that the result of the most recent operation is now available. Thus, *Call* allows to chain several delayed operations, without having to deal explicitly with actions or with the synchronization problems of threads. *Call* must be used to break up a procedure into a chain of asynchronously executed procedures whenever the procedure needs to directly manipulate the results of an asynchronous query, i.e. before the next *Read* is performed. (Several successive *Read*s on the same result table need not be broken up by *Call*, since once the result table has become available, it remains available unless a new query is started.)

The following example shows a chain of procedures linked by *Call* statements:

    PROCEDURE Last (t: SqlDB.Table; p: SqlDB.Par);

        VAR i: INTEGER;

    BEGIN    *(* t.Available() can be assumed *)*

        i := 0;

        WHILE t.res # SqlDB.noData DO

            t.Read(i, interactor);    *(* read *i*-th row into *interactor* *)*

            Out.Int(interactor.amount); Out.Ln;    *(* use interactor somehow *)*

            INC(i)

        END;

        t.Clear    *(* release table resources *)*

    END Last;

    PROCEDURE Middle (t: SqlDB.Table; p: SqlDB.Par);

    BEGIN    *(* t.Available() can be assumed *)*

        t.Read(0, interactor);

        interactor.amount := interactor.amount + 30;    *(* do something with interactor *)*

        t.Exec("SELECT * FROM SomeTable WHERE amount > 20");    *(* start next query *)*

        t.Call(Last, NIL)    *(* later continue with *Last* *)*

    END Middle;

    PROCEDURE First (t: SqlDB.Table; p: SqlDB.Par);

    BEGIN

        t.Exec("SELECT * FROM SomeTable WHERE someId = 42");    *(* start a query *)*

        t.Call(Middle, NIL)    *(* later continue with *Middle* *)*

    END First;

Note that the three procedures are given in reverse order, to avoid forward declarations. Parameter *p* can be used to pass arbitrary data along the chain of procedure invocations. Here it is sufficient to pass *NIL*.

This asynchronous mechanism is implemented by queueing database operations, with one queue per database object. This queue handles all *Exec* and *Call* calls (indepent of whether called via database or table object) in the strict order that they have been called originally. Only *table.Read* and *table.Clear* are optimized such that they may swap their places with operations that don't affect the corresponding table. As a result of this sequencing strategy, e.g., a *database.Commit* after a *table.Exec* always works correctly.

Sometimes it can be useful to have two database objects for the same database. For example, one may be opened synchronously, and the other asynchronously, when migrating from one to the other programming model. But this requires careful consideration of a possible interference between the two, since their individual chains of continuation procedures are not synchronized and may thus execute in any order, possibly resulting in different outcomes. Wherever possible, simultaneously opening to database objects on the same database should thus be avoided.

Obviously, asynchronous programming is more involved than synchronous operation. But if the database access is interactive and the database operation may involve a remote server, it is strongly suggested to consider the non-blocking asynchronous programming style. Blocking is considered "anti-social".

Even if a particular driver does not support true asynchronous operation, the asynchronous mode can still be used and will produce the same results, although possibly blocking sometimes. (*SqlOdbc* works this way.) But as soon as an asynchronous driver becomes available, the benefits will become apparent. For non-interactive applications, e.g., batch processing of a large database by night, there is no benefit in using asynchronous mode.

<a id="SqlDrivers"></a>**SqlDrivers**

File [<u>Sql/Mod/ObxDriv</u>](../Mod/ObxDriv.odc.md) contains a template of a new Sql driver. Its empty parts can be filled out to create a new driver for a database not yet supported by Sql, or to avoid the use of ODBC.

<a id="ODBC"></a>**ODBC driver (Windows only)**

Microsoft's Open Database Connectivity (ODBC) is an interface standard for accessing relational SQL databases. There are ODBC drivers for most relational products, and even for some non-relational databases.

*SqlOdbc* is an Sql driver which builds a bridge to the ODBC driver manager. Given a suitable 32-bit ODBC driver, this allows to use a database via ODBC and Sql.

*SqlOdbc* supports all features of Sql, except asynchronous operation.

To use ODBC, the ODBC driver manager from Microsoft and a suitable ODBC driver is needed. The ODBC driver is installed along with the first driver installed on a system. Consult the documentation of your database on how to install an ODBC driver for it.

During installation of BlackBox, it can be chosen whether the ODBC driver manager and one of Microsoft's "desktop drivers", the text driver, should be installed. The text driver uses plain ASCII text files as "database" files. It is very handy for testing purposes. It is not meant for productive use, and it is not permitted to distribute this Microsoft software along with applications.

In its software development kits, Microsoft also provides further desktop drivers, e.g., for Excel, Access,or dBase.

When connecting to a desktop driver, empty strings can be passed to the *password* and *id* parameters of *SqlDB.OpenDatabase*. The *datasource* parameter must be a name shown in the "User Data Sources" list in the ODBC control panel application. The *protocol* name must be the module name "SqlOdbc" (case sensitive!). Note that it is not possible to connect to a 16-bit driver.

    protocol    "SqlOdbc"

    datasource    data source name as presented in the ODBC control panel

    id    user id, if the ODBC driver supports user identification

    password    password, if the ODBC driver supports user identification

For productive use, Microsoft's SQL Server product is a suitable choice, not least because it directly implements ODBC, so no inefficient translation occurs in its driver, which slows down some other ODBC-connected database products.

*SqlOdbc* works with 32-bit ODBC drivers that support the ODBC core functions, ODBC Level 1, and the ODBC Level 2 procedure *SQLExtendedFetch*. It works with older ODBC managers. For ODBC 3, as it is distributed e.g. with Windows 2000, module *SqlOdbc3* must be used instead.

For further information on ODBC, please consult the Microsoft literature and software development kits.

<a id="SqlDB"></a>**SqlDB**

See the [<u>on-line documentation</u>](DB.odc.md).

<a id="Clearing"></a>**Clearing Database and Table pointers**

An open database need and cannot be closed explicitly. BlackBox closes it automatically when there exist no more references to it. This is accomplished by the garbage collector. When the garbage collector detects that a database object isn't accessible anymore, it finalizes (i.e., closes) it.

For the programmer it is important to make sure that no global database or table pointers are left which anchor the database object, and thus would prevent garbage collection.

For mere "batch" commands, this is no problem. Such a command is started, opens a database and possibly some tables, does some processing, and then terminates. Database and table pointers are kept as local variables, and thus vanish after the command has terminated. This means that there is no need to set any pointers to *NIL*, since they just disappear.

Things are different for interactive database applications, where the user interacts with a database via a dialog box. There the database should be opened when the (first) dialog box for it is opened, and closed after the (last) dialog box is closed. In between, there are references which keep the database from being collected.

Opening is simple: the menu entry which opens the dialog box first invokes an initialization command before opening the dialog box window. This initialization command opens the database and stores the necessary table objects in global variables. Typically, it won't save the database pointer itself; since most of the time a database ought to be closed when there exist no more table objects. Furthermore, a table's database can be accessed via its *base* field.

After the initialization command, other commands can use the database and table pointers to perform SQL queries and operations, until the user closes the last dialog box of the application.

The question is how the various global table pointers can be set to *NIL*, such that the garbage collector can perform its duty. For this purpose, module *SqlControls* provides so-called *anchor controls*. They are controls that can be linked to global table pointer variables. They have two main purposes: determining whether the window should be closed, and cleaning up if and when the window is closed.

An anchor control can be linked to a global table pointer variable. This is the actual "anchor" as far as the garbage collector is concerned. It can be "cut" (set to *NIL*) by the anchor control when the control's window is closed.

*Determining whether the window should be closed*

When the user attempts to close a dialog box containing one or several anchors, and if at least one of the anchors is linked to a non-NIL table variable (or has an empty link name), the user is asked whether the dialog box should really be closed. This is useful e.g. when the user has entered some data into a data entry mask, but hasn't performed an operation (e.g., a database insertion) with the data.

Asking only happens if the control's optional guard sets *disabled* to *TRUE*, and thus indicates that it wants to prevent closing the dialog box for the table (e.g., because entered data has not been inserted). If there is no guard, or if it doesn't set *disabled* to *TRUE*, then the user is not given the opportunity to stop window closing. But if the user is asked, he or she has the last word; an anchor control cannot prevent the user from closing the window.

The message that is shown to the user when asking is determined by the guard setting the *label* field. If there is no guard or it doesn't set this field, then the control's label value is used. If this label value is empty, then  a standard message (with a mapping defined in *Sql/Rsrc/Strings* with the key *IsCloseOK*) is used.

For example, a dialog box may contain two anchor views with the following properties:

link    "SqlObxDB.c"

label    "Do you really want to close the window without saving your input?"

link    "SqlObxDB.m"

label    ""

*Cleaning up when the window is closed*

If and when the dialog box is closed, all the anchors that it contains set their pointers to *NIL*. In other words: when the last anchor view vanishes, the table pointers are set to *NIL*. If there are no further references to the database object (directly or indirectly), then this means that it is not globally anchored anymore and can be garbage collected. Closing the window causes the garbage collector to run, so the database will be closed immediately.

If active notification about a closed database is desired, the control's notifier can be used for this purpose. This allows to perform arbitrary cleanup actions upon window closing, in addition to (or instead of) setting a table pointer to *NIL*.

The link may even be left empty, so that only the notifier is used and no pointer variable is needed.

*Control properties*

    link    optional    link to a global pointer variable

    label    optional    message that the user is asked before window closing

    guard    optional    determines whether to ask, and what message to show

    notifier    optional    notifies about window closing

Note that actually the anchor controls can be used universally for any kind of pointers, they need not be table pointers. Whenever you need to clear global pointers when no document references (i.e., anchors) to them exist anymore, you can use these anchors.

<a id="Displaying"></a>**Displaying tables**

Often it is necessary to display result tables in tabular fashion. For this purpose, module *SqlControls* provides *table controls*. Tables cannot be edited, but its possible to select a field in a table.

A table control needs to be linked to a global variable of type *SqlDB.Table*.

A table control may also denote a notifier with the following signature:

        TableNotifier = PROCEDURE (t: SqlDB.Table; row, column: INTEGER; modifiers: SET)

It is called whenever the user has clicked into a field of the table, indicating the table, its row and column numbers, and the track message's modifier set (-> Controllers.TrackMsg).

A table control is used in one of two ways: either it is opened in its own window, or it is embedded in some container, typically a form view. In the first case, the window provides scrollbars if necessary. In the second case, it is usually wrapped in a scroller view (Tools->Add Scroller) which provides the scrollbars.

When a table control is generated by a program, it is not necessary to link it to a global table pointer variable. Instead, the table pointer can be directly passed as parameter in the call to *SqlControls.dir.NewTableOn(table)*.

<a id="Design"></a>**Design rules**

This section gives some rules which should be followed when designing a database application. The reason for each rule is given after the rule.

Interactors or at least their types must be exported if they should be used as place holders in SQL statements.

(Non-exported types are not accessible through metaprogramming, and thus controls and the SQL string translation mechanism could not be used with them.)

A globally anchored database and all its table pointers must be set to *NIL* if the database ought to be closed.

(If a global pointer variable is not set to *NIL*, the garbage collector cannot reclaim the data structures anchored in it. Upon garbage collection of a database, the database is closed if there are no more pointers to it. Note that a table contains a pointer to its database object and thus anchors it.)

A database pointer should not be declared as global variable.

(Normally there are global table pointers, which contain references to their databases. A global database pointer would be one more pointer to set to *NIL* eventually; and anchor views can only set table pointers to *NIL.)*

*Database.Exec* may only execute non-row-returning statements, e.g. *DELETE* or *INSERT.*

(A result table must always be assigned to a table and via the table to an interactor, otherwise it cannot be accessed by the application.)

*Table.Exec* should only execute row-returning statements, e.g., *SELECT.*

The order and types (but not necessarily the names) of fields in an interactor variable must match those defined in the SQL database. The number of fields must be the same as the number of columns.

Only complete tables may be assigned to an interactor by *Table.Read*, no partial assignment is allowed. The correspondence of result table columns and interactor fields is given by their respective declaration order. Nested arrays, records and pointers are handled recursively.

Pointers must not be *NIL* except for POINTER TO ARRAY OF CHAR which are allocated automatically if the pointer is *NIL* or if the bound array is too small to receive the corresponding string.

Scalar result variables are treated as tables with *table.rows = 1* and *table.columns = 1*.

After an SQL statement which may render a row inconsistent with the database, use a *SELECT* statement to re-establish consistency.

(For example, an interactor may still contain the value of a row which has just been deleted through a *DELETE* statement. *Exec* on the table will flush the old result table and possibly assign a new one.)

<a id="Example"></a>**Example**

This very simple example allows to insert, update, delete, and find rows in a database. A row consists of an integer *id*, two strings *name *and *ceo*, and an integer *employees*. The application consists of a core logic module (*SqlObxDB*) and a user interface module with guards and notifiers (*SqlObxUI*).

[<u>SqlObxDB</u>](../Mod/ObxDB.odc.md)            [<u>SqlObxUI</u>](../Mod/ObxUI.odc.md)

Use the following menu entry to open the data entry mask (it may already be installed in the standard configuration of your software):

**MENU** "Sql"

    "Insert Anchor"    ""    "SqlControls.DepositAnchor; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    "Insert Table"    ""    "SqlControls.DepositTable; StdCmds.PasteView"    "StdCmds.PasteViewGuard"

    **SEPARATOR**

    "Browser..."    ""    "StdCmds.OpenAuxDialog('Sql/Rsrc/Browser', 'Browser')"    ""

    "Execute"    ""    "SqlBrowser.ExecuteSel"    "TextCmds.SelectionGuard"

    **SEPARATOR**

    "Company..."    ""    "SqlObxDB.Open; StdCmds.OpenAuxDialog('Sql/Rsrc/Company', 'Company')"    ""

    "Ownership..."    ""    "SqlObxExt.Open; StdCmds.OpenAuxDialog('Sql/Rsrc/Owner', 'Ownership')"    ""

    "Set Test Data"    ""    "SqlObxDB.SetTestData"    ""

    **SEPARATOR**

    "Help"    ""    "StdCmds.OpenBrowser('Sql/Docu/Dev-Man', 'Sql Docu')"    ""

**END**

The following form is opened when *Sql->Company...* is invoked:

 "StdCmds.OpenDoc('Sql/Rsrc/Company')"

On-line, there are some extensions to the above example. The module *SqlObxExt* demonstrates an extension of *SqlObxDB* and *SqlObxUI*. It handles the ownership dialog box, i.e., it allows to enter ownerships between companies. A suitable command to open the ownership dialog box is given in the menu definition above (*Sql->Ownership...*).

The modules *SqlObxGen, SqlObxViews, SqlObxNets* implement an algorithm which generates a graphical representation out of the companies in the sample database, showing the ownership relations of the companies. It is interesting insofar as this is a graph algorithm (known as Lee algorithm or Dijkstra's algorithm) which requires the creation of an intermediate data structure; fetching the necessary data cannot be formulated as a single SQL query.

To try this example out, find an existing company in the *Company* dialog box, e.g. company with id = 1 of the test data generated by *SqlObxDB.SetTestData*. Then click on the *Graph* button in the dialog box. This will cause *SqlObxGen.GenNet* to be called. This command implements a layout algorithm, which first finds all the companies which are related to the selected company by ownership, then converts the company and ownership information into a Component Pascal data structure (module *SqlObxNets*), then performs a layout algorithm to place the companies graphically, and then generates a view which displays the graph.

In order to run the examples provided by the Sql subsystem, some sample tables have to be created first. This can be done with the command *SqlObxInit.Setup*. The following two tables are created:

    CREATE TABLE Companies

        (id    INTEGER,

        name    CHAR(255)

        ceo    CHAR(255)

        employees    INTEGER)

    CREATE TABLE Ownership

        (owner    INTEGER,

        owned    INTEGER,

        percent    INTEGER)

Windows:

By default, the ODBC Text drivers are installed, for which the example database has already been created, so you don't need to perform the above CREATE TABLE statements anymore.

The modules *SqlObxDB*, *SqlObxExt*, *SqlObxGen*, and *SqlObxInit* contain ODBC-specific references (protocol, datasource, id, password) which have to be adapted before using them if another database driver is used instead of the *SqlOdbc* driver. It is sufficient to change the constant declarations appropriately.

The tables can be inspected and modified using the database browser *SqlBrowser*. Just type an SQL query in the *Statement* field of the browser window and press return.

