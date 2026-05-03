<a id="7.7"></a>**ComConnect Example**

This example shows how connectable objects are implemented using the COM compiler. The example is described in detail in chapter four of the book "Inside OLE". Connectable objects offer an interface which allows to install interfaces which can be called back by the connectable object.

A connectable object has to implement the interface *IConnectionPointContainer *with the two methods *FindConnectionPoint *and* EnumConnectionPoints. *With the first method, the connectable object can be asked for a specific outgoing interface. If the interface is supported, then a pointer to the *IConnectionPoint* is returned, the interface through which the callback interface can be installed. With the method *EnumConnectionPoints *the supported outgoing interfaces can be requested. The list of supported interfaces is returned in the form of an enumeration interface, *IEnumConnectionPoints*.

In our example, the connectable object keeps a list of the supported interfaces as an instance variable.

The *IConnectionPoint* interface allows to establish a connection between the connectable object and the client through the *Advise *method. A cookie (key) is returned as result. A connection point can support any number of connections. In our example, the constant *connMax* defines the number of connections that can be established. With the *Unadvise* method a connection can be terminated. The connection cookie must be passed as argument.

As argument of the *Advise* method the sink object's *IUnknown* interface is passed. From this interface the outgoing interface is requested and stored in the connection (the object implementing *IConnectionPoint*).

Besides these two methods, the *IConnectionPoint* interface supports the methods *EnumConnections*, *GetConnectionPointContainer* and *GetConnectionInterface*. The implementation of the latter two functions is straight-forward. The *EnumConnections* function returns an enumeration of all the currently installed connections. This enumeration interface is implemented with *CEnumConnections*.

The implementation of *IConnectionPoint *has the following instance variables:

        CConnectionPoint = POINTER TO RECORD (WinOle.IConnectionPoint)

            obj: CConnObject;

            iid: COM.GUID;

            unk: ARRAY connMax OF COM.IUnknown;

            cookies: ARRAY connMax OF INTEGER;

            conn: INTEGER;

            next: INTEGER

        END;

The *obj* reference refers to the connectable object, i.e. to the object implementing the* IConnection-PointContainer* interface.

The GUID of the outgoing interface, which is supported by the concrete connection, is stored in the field *iid*. In our example this is always COM.GUID(IDuckEvents).

All installed connections are stored in the *unk* array. If an event is distributed through all conected interfaces, then the necessary interface is requested though the *IUnknown* interfaces installed in the *unk* array.

The cookies corresponding to the connections are stored in the *cookies* array. They are necessary to search a connection when *Unadvise* is called.

The *conn* field counts the number of installed connections, and the *next* field finally contains the last distributed *cookie*.

The *IDuckEvents* interface is used in this example as the outgoing interface. This interface supports the following three functions:

        IDuckEvents = POINTER TO ABSTRACT RECORD

                ["{00021145-0000-0000-C000-000000000046}"] (COM.IUnknown)

            (this: IDuckEvents) Quack (): COM.RESULT, NEW, ABSTRACT;

            (this: IDuckEvents) Flap (): COM.RESULT, NEW, ABSTRACT;

            (this: IDuckEvents) Paddle (): COM.RESULT, NEW, ABSTRACT;

        END;

In its instance variable, the concrete implementation of this interface keeps an identifier and the cookie which is obtained when the interface is installed in the connectable object. The implementation of the methods write a text to the log indicating that they have been called. They also write their identifier into the log.

In our example, the connectable object *CConnObject* offers the additional method *TriggerEvent. *With this method, an event can be sent through all registered *IDuckEvents* interfaces.

The module *ComConnect* offers the following commands

Init    creates two objects which implement the *IDuckEvents* interface

Release    disconnects all connections and frees the connectable object

        and the sink objects

ObjectCreate    creates the connectable object

ObjectRelease    releases the connectable object

SinkXConnect    connects a sink object to the connectable object

SinkXDisconnect    disconnects a sink object from the connectable object

TriggerQuack    calls the *Quack* method of all connected *IDuckEvents* interfaces

TriggerFlap    calls the *Flap* method of all connected *IDuckEvents* interfaces

TriggerPaddle    calls the *Paddle* method of all connected *IDuckEvents* interfaces

This example is coded rather in a general way. It would be easy to extend the implementation to support more connections or more outgoing interfaces. If only one outgoing interface were supported, the code could be simplified at several places.

<u>[ComConnect  sources</u>](../Mod/Connect.odc.md)

