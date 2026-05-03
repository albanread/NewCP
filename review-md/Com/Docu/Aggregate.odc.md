<a id="7.8"></a>**ComAggregate Example**

This example shows how aggregation is implemented using the COM compiler. The example is described in detail in chapter two of the book "Inside OLE".

With aggregation, an object which implements one or several interfaces is reused without modification and without the need to forward each method of the reused interfaces. The reused object is called the inner one, and the object which reuses the inner object is called the outer object. The inner object delegates all calls to *QueryInterface* to the outer object, and the outer object has a reference to the inner object's *IUnknown* interface. This latter interface must not forward the *QueryInterface* calls, otherwise this would produce an infinite recursion. This implies that an aggregatable object must provide a separate implementation of the *IUnknown* interface, i.e. an aggregatable object consists of at least two records which both implement the *IUnknown* interface, where one *QueryInterface *method is forwarded, while the other is not. These two records will be linked by references from one to the other.

The interface which is reused has the *IAnimal* interface,

        IAnimal = POINTER TO ABSTRACT RECORD

                                            ["{00021143-0000-0000-C000-000000000046}"] (COM.IUnknown)

            (this: IAnimal) Eat (): COM.RESULT, NEW, ABSTRACT;

            (this: IAnimal) Sleep (): COM.RESULT, NEW, ABSTRACT;

            (this: IAnimal) Procreate (): COM.RESULT, NEW, ABSTRACT;

        END;

and the two objects which implement the aggregatable implementation of this interface are defined below.

        CAnimal = POINTER TO RECORD (COM.IUnknown)

            impl: CImpIAnimal

        END;

        CImpIAnimal = POINTER TO RECORD (IAnimal)

            obj: CAnimal

        END;

The *CAnimal* record implements the *IUnknown* interface and the *CImplAnimal* record implements the *IAnimal* interface. If the object is aggregated, the calls to the *QueryInterface* method of the latter record are forwarded to the outer object.

Since objects are created inside-out, the outer object is passed as a parameter to the procedure which creates an *IAnimal* object. If the reference to the outer object is not NIL, then the *IAnimal* object is aggregated. It is a convention of COM that then the *IUnknown* interface of the aggregated object must be requested upon object creation.

As the outer object has a reference to the inner one, and the inner one a reference to the outer one, there is a cycle of references which would prevent garbage collection. To avoid this, the reference from the inner to the outer object is a special reference which is not reference counted. Such references can be generated with the two-argument NEW. The default implementation of *QueryInterface* then automatically forwards the requests to the outer object.

If the object is not aggregated, we use the *CAnimal* record as "outer" object for the *CImplAnimal* object. This is the same pattern as in the ComObject example, i.e. a COM object which implements several interfaces.

The whole procedure to create an *IAnimal* object is given below. The first parameter (outer) determines whether the object is aggregated or not.

    PROCEDURE CreateAnimal (outer: COM.IUnknown; IN [iid] iid: COM.GUID;

                                        OUT [new] int: COM.IUnknown): COM.RESULT;

        VAR new: CAnimal;

    BEGIN

        IF (outer # NIL) & (iid # COM.ID(COM.IUnknown)) THEN

**            RETURN** WinApi.CLASS_E_NOAGGREGATION

        END;

        NEW(new);

        IF new # NIL THEN

            IF outer = NIL THEN NEW(new.impl, new)

            ELSE NEW(new.impl, outer)

            END;

            IF new.impl # NIL THEN

                new.impl.obj := new;

                StdLog.String("Animal allocated"); StdLog.Ln;

            **    RETURN** new.QueryInterface(iid, int)

            END

        END;

        **RETURN** WinApi.E_OUTOFMEMORY

    END CreateAnimal;

The inner object's explicit *IUnknown* interface must control the inner object's reference count and implement the *QueryInterface *behavior for only the inner object. The procedure which is doing this has the following form:

    PROCEDURE (this: CAnimal) QueryInterface (IN [iid] iid: COM.GUID;

                                                                                OUT [new] int: COM.IUnknown): COM.RESULT;

    BEGIN

        IF COM.QUERY(this, iid, int) OR COM.QUERY(this.impl, iid, int) THEN **RETURN** WinApi.S_OK

        ELSE **RETURN** WinApi.E_NOINTERFACE

        END

    END QueryInterface;

Note that the COM.QUERY command on the *CImplAnimal* interface is not forwarded. It succeeds if the *IAnimal* interface is requested.

Note that in the example, the outer object which implements the *IKoala* interface is also designed to support aggregation. The object is separated into two records, one implementing the *IUnknown* interface and the other the *IKoala* interface. This design is easy to extend if additional interfaces have to be supported.

The example finally provides some commands which allow to test the functionality of the IKoala-IAnimal implementation.

<u>[ComAggregate  sources</u>](../Mod/Aggregate.odc.md)

