**CommStreams**

DEFINITION CommStreams;

    CONST

        done = 0;

        noSuchProtocol = 1;

        invalidLocalAdr = 2;

        invalidRemoteAdr = 3;

        networkDown = 4;

        localAdrInUse = 5;

        remoteAdrInUse = 6;

    TYPE

        Adr = POINTER TO ARRAY OF CHAR;

        Listener = POINTER TO ABSTRACT RECORD

            (l: Listener) Accept (OUT s: Stream), NEW, ABSTRACT;

            (l: Listener) Close, NEW, ABSTRACT;

            (l: Listener) LocalAdr (): Adr, NEW, ABSTRACT

        END;

        Stream = POINTER TO ABSTRACT RECORD

            (s: Stream) Close, NEW, ABSTRACT;

            (s: Stream) IsConnected (): BOOLEAN, NEW, ABSTRACT;

            (s: Stream) ReadBytes (VAR x: ARRAY OF BYTE; beg, len: INTEGER;

                                                OUT read: INTEGER), NEW, ABSTRACT;

            (s: Stream) RemoteAdr (): Adr, NEW, ABSTRACT;

            (s: Stream) WriteBytes (IN x: ARRAY OF BYTE; beg, len: INTEGER;

                                                OUT written: INTEGER), NEW, ABSTRACT

        END;

    PROCEDURE NewListener (protocol, localAdr: ARRAY OF CHAR; OUT l: Listener;

                                                OUT res: INTEGER);

    PROCEDURE NewStream (protocol, localAdr, remoteAdr: ARRAY OF CHAR; OUT s: Stream;

                                                OUT res: INTEGER);

END CommStreams.

Module *CommStreams* defines the interfaces of two types: *Stream* and *Listener*. The interface is designed with reliable connection-oriented protocols in mind. Implementations of the stream and listener interfaces provide the actual communication channels. A TCP/IP implementation which internally uses a "sockets" interface is a typical example. Only implementors, not clients, need to extend the *Stream* and *Listener* types. In the following text it is distinguished between clients and implementors of these interface types.

The stream interface is non-blocking. The reason for this approach is the non-modal nature of the BlackBox Component Framework. Letting one component block the whole system (and thus the user) would be contrary to the framework's basic design goals. Implementing non-blocking communicating objects is easy, although unconvential. Fortunately, object-oriented programming lends itself to a straight-forward implementation of such objects.

**Specification for clients**

*Streams* are full-duplex synchronous communication channels. They are used through read and write procedures. These procedures *never* block. The read procedure reads as much data as possible and then returns immediately. The write procedure writes as much data (into an internal buffer) as possible and then returns immediately. The number of bytes read or written is returned as a result of the procedures.

Note that depending on the used protocol, individual read or write operations can have considerable overhead. This means that transferring entire groups of data in one step whenever possible can lead to a considerable performance increase, compared to sending the same data in several steps.

There exist two ways to obtain a new object of type *Stream*, which represents a newly created connection. If a connection is to be made to a partner awaiting requests ("server"), the procedure *NewStream* is used. As parameters the name of the protocol to be used (or more exactly, it's driver module's name), and address identifiers for the remote port and for a local port are passed. The format of these identifiers, which are passed as character strings, depend on the protocol that is used.

If the possibility to listen for calls from another site ("client") is to be offered, a listener object is created first. Such a listener is used to test whether a connection request has been made by another site. Whenever this is the case, a new *Stream* object can be obtained from the listener. The listener itself can be used further until it is explicitly closed.

**Requirements for implementors**

Implementors of the interface have to implement all the procedures of the types *Stream* and *Listener* according to the specification described in the previous section. Furthermore, two procedures for object allocation must be exported, with the following signatures:

PROCEDURE NewStream (localAdr, remoteAdr: ARRAY OF CHAR; OUT s: Stream; OUT res: INTEGER)

PROCEDURE NewListener (localAdr: ARRAY OF CHAR; OUT l: Listener; OUT res: INTEGER)

These procedures will be activated by *CommStreams* using the metaprogramming facility of BlackBox. For example, if the protocol is TCP/IP, its driver's module name is used as protocol name, e.g., "CommTCP". For creating a stream, "CommTCP.NewStream" is called with the above described arguments. For creating a listener, "CommTCP.NewListener" is called with its set of arguments.

CONST **done**

Predefined value for parameter *res* of *NewStream* and *NewListener*. Signals that the operation could be carried out.

CONST **noSuchProtocol**

Predefined value for parameter *res* of *NewStream* and *NewListener*. Signals that the module specified as *protocol* could not be found, could not be loaded, or does not implement the required interface.

CONST **networkDown**

Predefined value for parameter *res* of *NewStream* and *NewListener*. Signals that the network resource used by the specified *protocol* is not available and hence no connections can be made.

CONST **invalidLocalAdr**

Predefined value for parameter *res* of *NewStream* and *NewListener*. Signals that the value specified as *localAdr* could not be used; either because the string was syntactically invalid, or because the specified address does not exist.

CONST **localAdrInUse**

Predefined value for parameter *res* of *NewStream* and *NewListener*. Signals that the value specified as *localAdr* could not be used because the address is in use already.

CONST **invalidRemoteAdr**

Predefined value for parameter *res* of *NewStream*. Signals that the value specified as *remoteAdr* could not be used; either because the string was syntactically invalid, or because the specified address does not exist.

CONST **remoteAdrInUse**

Predefined value for parameter *res* of *NewStream*. Signals that the value specified as *remoteAdr* could not be used because the address is in use already.

TYPE **Adr**

Dynamic string type for names of local or remote addresses.

TYPE **Listener**

ABSTRACT

Objects of type *Listener* offer services to the network.

PROCEDURE (l: Listener) **Accept** (OUT s: Stream)

NEW, ABSTRACT

Creates a new *Stream* when a remote site requests a connection. *Accept* never blocks: either it returns a new stream, or *NIL* if no service was requested from the network. If the listener is closed, *Accept* always returns *NIL*.

Post

s = NIL

    no service was requested

s # NIL

    service was requested over the network

PROCEDURE (l: Listener) **Close**

NEW, ABSTRACT

Closes the listener, so that no services can't be requested over the network anymore. *Close* has no effect if the listener was closed already.

PROCEDURE (l: Listener) **LocalAdr**

NEW, ABSTRACT

Returns the name of the local port on which the listener listens.

TYPE **Stream**

ABSTRACT

Objects of type *Stream* represent full-duplex connections.

PROCEDURE (s: Stream) **Close**

NEW, ABSTRACT

Closes the connection. If the connection was already closed, *Close* has no effect.

Post

~s.IsConnected()

PROCEDURE (s: Stream) **IsConnected** (): BOOLEAN

NEW, ABSTRACT

Tests whether connection is still open, or whether it has been closed by either partner of the connection. No data can be read anymore.

*IsConnected* is not precise in time. It only represents the current knowledge of the stream implementation about the state of the connection. In reality, this state may already have changed. This means that *IsConnected* may still return *TRUE* even if the connection has already been closed down.

Post

result = FALSE

    connection has been closed by one of the partners

result = TRUE

    connection is probably still open

PROCEDURE (s: Stream) **ReadBytes** (VAR x: ARRAY OF BYTE; beg, len: INTEGER;

                                                                OUT read: INTEGER)

NEW, ABSTRACT

Receives up to *len* bytes from the communication link and reads them. The procedure never blocks. No bytes are read if *~s.IsConnected()*.

Pre

beg >= 0    20

len > 0    21

LEN(x) >= beg + len    22

Post

read >= 0

x[beg ... beg + read[ = bytes read

PROCEDURE (s: Stream) **WriteBytes** (IN x: ARRAY OF BYTE; beg, len: INTEGER;

                                                                OUT written: INTEGER)

NEW, ABSTRACT

Sends up to *len* bytes over the communication link. The procedure never blocks. How many bytes are written might depend on internal resources, like buffer size. However, no guarantee of reception of the data by the partner site is given. No bytes are written if *~s.IsConnected()*.

Pre

beg >= 0    20

len > 0    21

LEN(x) >= beg + len    22

Post

written >= 0

x[beg ... beg + written[ = bytes written

PROCEDURE  **NewListener** (protocol, localAdr: ARRAY OF CHAR; OUT l: Listener; OUT res: INTEGER)

Offers connections on the port specified by *localAdr *using the transport service *protocol*. The format of *localAdr* depends on the protocol used.

For TCP/IP, *protocol = "CommTCP"* and *localAdr* is the local port number (may be the empty string).

Result codes (parameter *res*) may be one of the following (see *const* definitions above) or a protocol-specific value:

    done, noSuchProtocol, networkDown,  invalidLocalAdr, localAdrInUse

Pre

protocol # ""    20

Post

res = 0

    l # NIL

    listener is open

res # 0

    l = NIL

PROCEDURE  **NewStream** (protocol, localAdr, remoteAdr: ARRAY OF CHAR;

                                                OUT s: Stream; OUT res: INTEGER)

Connects to the site and port specified by *remoteAdr *using the local port given by *localAdr.* The format of the addresses depend on the protocol used. For example, for *protocol = "CommTCP"*, *localAdr* must be a port number, e.g., 4. If the empty string is passed, port number 0 ("don't care") is used. The *remoteAdr* must either be an IP address such as "100.0.0.7", or the host's name. Either of them may optionally be followed by a ":" and a port number, e.g., "100.0.0.7:2". Without this option, port number 0 is assumed.

For TCP/IP, *protocol = "CommTCP"* and *localAdr* is the local port number (may be the empty string).

Result codes (parameter *res*) may be one of the following (see *const* definitions above) or a protocol-specific value:

    done, noSuchProtocol, networkDown, invalidLocalAdr, localAdrInUse, invalidRemoteAdr, remoteAdrInUse

Pre

protocol # ""    20

Post

res = 0

    s # NIL

    stream is open

res # 0

    s = NIL

