**CommTCP**

Driver for *CommStreams*.

It implements TCP/IP streams, using the operating system's sockets interface. It provides the *NewListener* and *NewStream* factory functions as needed for *CommStreams*. Do not import *CommTCP* directly; instead, use *CommStreams* and specify "CommTCP" as the *protocol* name.

**Parameters *****localAdr***** and *****remoteAdr*****:**

The *CommStreams* procedures *NewStream* and *NewListener* feature string parameters *remoteAdr* and *localAdr*. The interpretation of the strings passed depends on the implementation of the actual driver module. For *CommTCP* the following holds.

The *remoteAdr* (only used with *NewStream*) must be either an IP address or a host's name, followed by a colon (":") and a port number. (Examples: "127.0.0.1:2", "loopback:2".)

The parameter *localAdr* identifies the port on which a listener is established (*NewListener*) or from which a remote connection is attempted (*NewStream*). Valid values are

-    an empty string

-    an IP address or a host's name (Examples: "127.0.0.1", "loopback")

-    an IP address or a host's name followed by a colon (":") and a port number

(Examples: "127.0.0.1:2", "loopback:2")

-    a port number (Example: "2")

For unspecified parts, IP addresses / host's names or port numbers respectively, a wildcard is used, directing the operating system to choose freely. If in a call to *NewListener* neither IP address nor host name is specified for *localAdr*, the listener will check on all IP numbers available on the local machine.

Note: Certain operating systems (e.g. Microsoft Windows Server 2003) do not accept an empty string or just a port number as valid values for *localAdr*. Use the values "0.0.0.0:0" or "0.0.0.0:*<port number>*" (e.g. "0.0.0.0:2") instead.
