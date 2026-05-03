**Part I: Design Patterns**

Part I of the BlackBox tutorial gives an introduction to the design patterns that are used throughout BlackBox. To know these patterns makes it easier to understand and remember the more detailed design decisions in the various BlackBox modules.

Part II of the BlackBox tutorial demonstrates how the most important library components can be used: control, form, and text components.

Part III of the BlackBox tutorial demonstrates how new views can be developed, by giving a series of examples that gradually become more sophisticated.

**1 User Interaction**

In this section, we will discuss how a user interface can be made user-friendly. Graphical user interfaces are part of the answer. Unfortunately, this answer leads to another problem: how can such a user interface be implemented at reasonable cost? The design approach which solves this problem is surprisingly fundamental, and has deep consequences on the way we construct modern software. It is called object-oriented programming.

**1.1 User-friendliness**

There are at least 100 million personal computers in use today. With so many users who are not computer specialists, it is important that computers be easy to use. This implies that operating systems and applications must be user-friendly. What does this mean? There are several aspects.

First, the various applications should be consistent with one another: a function that is available in several programs should have the same user interface in all of them, so that the user does not have to learn different ways to do one and the same thing. One way to achieve this is to publish user interface guidelines that all software developers should adhere to. This approach was championed by Apple when they introduced the Macintosh. A second approach is to implement a function only once, so that it can be reused across different applications. This is the idea of component software. Apple didn't go that far, but they at least provided a number of so-called "toolbox" routines. The toolbox is a comprehensive library built into the Mac OS.

Second, the idea of the desktop metaphor, where icons graphically represent files, directories, programs, and other items, made it possible to replace arcane commands that have to be remembered and typed in by more intuitive direct manipulation: for example, an icon can be dragged with the mouse and dropped into a waste basket, instead of typing in "erase myfile" or something similar. Such a rich user interface requires a vast set of services. Not surprisingly, the Mac OS toolbox included services for windows, menus, mouse, dialog boxes, graphics, and so on. The toolbox made it possible to build applications that really adhered to the user interface guidelines. Unfortunately, programming for such an environment implied a tremendous leap in complexity. As a result, the Macintosh was much easier to use than earlier computers, but it was also much more difficult to program.

Today, the browser metaphor is an increasingly popular addition to traditional graphical user interfaces. Combined with Internet access, it opens a huge universe of documents to a computer user - and makes the job of a developer even more complicated.

Finally, one of the most general and most important aspects of user-friendliness is the avoidance of modes. A modal user interface separates user interactions into different classes, and gives a separate environment to each class. Switching between environments is cumbersome. For example, in a database application, a user may open a window (or "screen", "mask") for data entry. In this window, no records may be deleted, no records may be searched, no notes taken, no email read, etc. For each of these other tasks, the data entry environment would have to be left first. This is cumbersome, and reminds us of the application- rather than the document-centric way of computing discussed in the first part of this book. In complex applications, it is often difficult to remember where you currently are, what commands are available, and where you may go next. Figure 1-1 shows an example of the states and possible state transitions of a hypothetical modal application.

Figure 1-1. Navigation possibilities in a modal user interface

Non-modal user interfaces have no separate working environments, or at least let you switch between environments without losing the state of others. For example, if during data entry you want to look up something with a library browser or text searching tool, you don't lose the partial data that you have already entered. If there are separate working environments,  navigation is made simple and obvious by giving explicit feedback about the environment and its capabilities.

A modern graphical user interface is a prime example of a mostly non-modal user interface. There may be several windows (environments) open at the same time, but it is clearly visible with which window you are currently interacting (usually the "top" window). You can work in one window, temporarily work in another one (thereby bringing it to the top), and later switch back to resume your work in the first window.

These user interfaces are a huge improvement over old-style modal interfaces. Users feel more comfortable because they have a better idea of where they are and what they can do. However, even modern GUIs are far from perfect. Most of them still use modal dialog boxes, i.e., windows that you can't put aside to work with another one for a while. For example, a typical modal file dialog box lets you open a file, but doesn't allow you to quickly switch to a file searching tool and back again. The dialog box "modes you in". Even the fact that you have to bring a window to the top before manipulating it constitutes an inconvenient and unnecessary mode, at least if you have a screen large enough that you can lay out all needed windows in a non-overlapping fashion. And many database products don't allow you to have several data entry or data manipulation forms open at the same time. A Web browser is the closest thing to a truly non-modal user interface today: if you have entered some incomplete data into an HTML form, and then switch to another Web page, the browser won't prevent you from doing it.

The BlackBox Component Builder is more radical than other tools in that it simply doesn't support modal dialog boxes. Every dialog box is non-modal, except for a very few built-in operating system dialog boxes such as the standard file dialog boxes. In general, you can always put dialog boxes aside and get back to them later.

**1.2 Event loops**

Implementing a non-modal application looks deceptively simple: the program waits for a user event, such as a mouse click or a key press; reacts in an appropriate way; waits for the next event; and so on. This programming style, with a central loop that polls the operating system for events and then calls the appropriate handlers for them, is called event-driven programming.

Figure 1-2. Event loop of a non-modal program

The event-polling loop of a typical event-driven program looks similar to the following program fragment:

    PROCEDURE Main;

        VAR event: OS.Event;

    BEGIN

        LOOP

            event := OS.NextEvent();    *(* poll the operating system for the next event *)*

            IF event IS KeyDownEvent THEN

                HandleKeyDown(event)

            ELSIF event IS MouseDownEvent THEN

                IF MouseInMenu(event) THEN

                    HandleMenuEvent(event)

                ELSIF MouseInWindowTitle(event) THEN

                    HandleTitleEvent(event)

                ELSIF MouseInWindowBorder(event) THEN

                    HandleBorderEvent(event)

                ELSIF MouseInWindowContents(event) THEN

                    HandleContentsEvent(event)

                ELSIF...

                    ...

                END

            ELSIF ...

                ...

            END

        END Main;

Listing 1-3. Event loop with cascaded selection of event handlers

In realistic programs, these cascaded IF-statements can easily span several pages of code, and are always similar. Thus it suggests itself to put this code into a library (or even better, into the operating system).

Such a library would implement the standard user interface guidelines. For example, the *HandleTitleEvent* would further distinguish where in the window's title bar the mouse has been clicked. Depending on the exact location, the window will be dragged to another place, zoomed, closed, etc. This is a generic behavior that can, and for consistency reasons should, only be implemented once. A procedure like *HandleContentsEvent* is different, though. A standard library cannot know what should happen when the user has clicked into the interior (contents) area of a window.

If we require that the library itself must never need adaptation to a particular application, then there results a peculiar kind of system structure, where a library sometimes calls the application. Such a call is called a "call-back". When we visualize an application as sitting on top of the library, it becomes obvious why such calls are also called "up-calls" and the resulting programming style as "inverted programming":

Figure 1-4. Inverted programming design pattern

A library which strongly relies on inverted programming is called a *framework*. It is a semi-finished product that must be customized by plugging in procedures at run-time. For some of the procedures, this can be optional: for procedure *HandleTitleEvent* there can be a default implementation that implements the standard user interface guidlines. Such procedures will only be replaced if unconventional behavior is desired.

Experience shows that a procedure plugged into a framework frequently refers to the same data. For example, the procedure *HandleTitleEvent* will often access the window in which the user has clicked. This makes it convenient to bundle the procedure and its state into a capsule. Such a capsule is called an *object*, and it lifts inverted programming to *object-oriented programming*.

Thinking about user-friendly software led us to the problem of how to reduce the amount of repetitive and complex coding of event loops, which led us straight to the inverted programming design style; to frameworks as a way of casting such a design style into code; and to object-oriented programming as a means to implement frameworks in a convenient way.

The BlackBox Component Framework hides the event loop from application programmers. It goes even further than older frameworks in that it also hides platform-specific user-interface features such as windows and menus. This was achieved by focusing on the abstraction that represents the contents of a window: the so-called *View* type. We will come back to this topic in Chapter 2 in more detail.

**1.3 Access by multiple users**

Personal computers have made computer users independent from central IT (information technology) departments, their bureaucracies, and their overloaded time-sharing servers. This independence is a good thing, if it can be combined with integration where this is useful and cost-effective. Local area networks, wide area networks, and then the Internet have made integration possible. When two computers cooperate over a network, one of them asks the other to provide a service, for example to deliver a file over the network. The computer which issues the request is called the *client*, the other one is called the *server*. The same machine may act both as a client and as a server, but often these roles are assigned in a fixed manner: the clerk at the counter of a bank's branch office always uses a client machine, and the large box in the air-conditioned vault of the bank's headquarters is always used as a server. A bank's internal network may connect thousands of clients to dozens of servers. But even the smallest network obviously requires that applications are split into two parts: a client and a server part. Consequently, this kind of architecture is called *client/server* computing. It allows to assign processing tasks to the machines which are most suitable. In enterprise environments, the most popular assignment is to put a centralized database on a *database server*, and the remaining functionality on the client machines. This is called a *2-tier* architecture. It requires *fat clients*, i.e., client machines have to perform everything but the actual database accesses. To reduce this burden, high-end database management systems allow to execute some code on the server itself, as an interpreted *stored procedure*. This can greatly increase performance, because it can prevent large amounts of data being shuffled back and forth over the network.

If large numbers of clients are involved, it can become hard to keep all client installations up-to-date and consistent. This problem can be reduced using a *3-tier* architecture, where special application servers are interposed between clients and servers. The "thin" clients are reduced to implementing user interfaces, the database servers handle the databases, while the application servers contain all application-specific knowledge ("business logic").

Figure 1-5. 3-tier client/server architecture pattern

A 3-tier architecture is reasonably manageable and scalable in terms of size. Note that a software system separated in this way can be scaled down, in the extreme case by putting the client, application server, and database server on a single machine. The other way around does not work: a monolithic application cannot be easily partitioned to fit a client/server architecture.

Clients and servers are coupled over a network. Communication either operates at the low abstraction level of untyped byte streams (e.g., using a sockets interface for TCP/IP communication) or at the high abstraction level of typed data. In the latter case, either distributed objects (for immediate point-to-point communication) or message objects (for multicast or delayed communication) are used.

The *Comm* subsystem of the BlackBox Component Framework provides a simple byte stream communication abstraction, on top of which messaging services can be built. Among other things, this has been used to implement remote controls, i.e., controls which visualize and manipulate data on a remote machine. For example, the internal state of an embedded system can be monitored in this way.

The *Sql* subsystem of the BlackBox Component Framework provides a simple distributed object interface specialized for accessing SQL databases. More general DCOM-based [COM] distributed objects can be accessed and implemented with the optional Direct-To-COM Component Pascal compiler.

**1.4 Language-specific parameters**

The global nature of today's software business often makes it necessary to produce "localized" versions of a software package. At the least, this requires translation of all string constants in the program that the user may see. For example, in a country like Switzerland, where four official languages are spoken, it is often required that the same application is available in German, French, and Italian language versions. This can even mean that the layouts of dialog boxes need to be adapted to the different languages, since the captions and control labels have different lengths in different languages.

If such language-specific aspects are hard-coded into the program sources, language-specific versions require editing and compiling the source code. This is always a sensitive point, since errors and inconsistencies can easily be introduced. It is also very inconvenient: hard-coded layouts of dialog boxes are very unintuitive to modify, and the edit-compile-run cycle is cumbersome if you only want to move or resize a control.

Many tools have been developed which provide more convenient special-purpose editors, in particular layout editors for dialog boxes. One type of tool produces source code out of the interactively produced layout, which then has to be compiled. Since a programmer have to edit the generated source code, it is easy to introduce inconsistencies between the layout and the source code.

Fortunately, there is a much better way to solve this problem, by avoiding the intermediate source-code generator and directly use the editor's output in the program. The editor saves the dialog box layout in a file, a so-called *resource* file. The program then reads these resources when it needs them, e.g., when it opens a dialog box. Resources can be regarded as persistent objects which are only modified if the configuration is changed.

We have seen that the problem of convenient adaptation of a program to different languages can be solved by separating language- or location-specific parameters from the source code. They are put into resource files, which can be modified by convenient special-purpose editors.

Moving user-interface aspects away from the proper application logic is a way to make client software more modular and better adaptable. If taken to the extreme, resources almost *become* the client software: a Web browser on a client is all that is needed to present a user interface; the resources (HTML texts) are downloaded from a server whenever needed. (However, this approach with "ultra-thin" clients becomes less credible the fatter the Web browsers themselves become.)

The BlackBox Component Framework uses its standard document format to store resources. For example, a dialog box and its controls are simply parts of a compound document stored in a file. The same view is used for layouting and for actually using a dialog box - no separate layout editor progam is needed. Resources can be available in several language versions simultaneously, and languages can even be switched at run-time. This is useful for customs applications or other programs used in regions where several languages are spoken.

