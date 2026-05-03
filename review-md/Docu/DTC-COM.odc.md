** Direct-To-COM Compiler**

<a id="2"></a>**The Component Object Model**

The Component Object Model (COM) is an object model which allows independently developed binary software components to connect to and communicate with each other in a well-defined manner. COM defines a binary standard for object interoperability which is programming-language and compiler independent.

The primary architectural feature of COM is that an object exposes a set of interfaces to a client. An interface is a set of semantically related functions. Every object can provide multiple interfaces. COM objects can only be accessed through pointers to interfaces (interface pointers). Direct access to an object's internal variables is not possible. This allows for encapsulation of data and processing, a fundamental requirement of a true component software standard.

Another fundamental concept of the COM model is reference counting. Reference counting allows objects to track their own lifetime and delete themselves when appropriate.

The following figure visualizes a COM object as large black box, which is accessible from the outside only via pointers to its exported interfaces. Internally, a COM object typically contains a large number of simpler objects, which constitute its hidden implementation. Note that there are pointers from each interface record of the COM object to each other interface record, directly or indirectly. Also, every implementation object is connected to the interfaces through one or several pointer chains. Pointer chains may contain cycles.

<a id="2.1"></a>**Binary Standard**

COM defines a standardized way to layout the concrete implementation of interfaces. Every COM object provides the implementation of each method specified in the interface. The pointers to the method implementations are stored in an array, a so called virtual function table (vtable). The interface record is a structure whose first entry is a pointer to a vtable; it may contain additional private object data. COM clients only interact with a COM object through pointers to interfaces, i.e. with pointers to pointers to vtables. It is through such a pointer that the client accesses the object's implementation of the interface. Therefore, any language that can call functions via pointers (using the *StdCall* calling conventions) can be used to write and use COM components.

    interface pointer    interface    vtable

<a id="2.2"></a>**Reference Counting in COM**

COM uses reference counting as a simple form of (manual) garbage collection. Reference counting is performed through two standard methods called *AddRef* and *Release*. Whenever a new reference to an interface is established, *AddRef* must be called. When the interface is no longer needed, *Release* must be called. Reference counting is done at the interface level, i.e., every interface must support the two methods *AddRef* and *Release*. They are defined in the interface *IUnknown* which is the base interface of every COM interface. Internally, the interface implementation counts the number of *AddRef* and *Release* calls, and thereby knows when it is safe to free the memory that it occupies.

Conceptually, reference counting is performed at the interface level. However, as a COM object can support several interfaces, it is free to implement one central reference count per object, instead of one reference count per interface. However, a client should never assume that an object uses the same counter for several interfaces; i.e., it should increment the reference count always through the interface which is about to be used.

The following two simple rules define how reference counts have to be managed. Interface pointer variables are variables which hold interface pointers. Such variables may be located in memory or in processor registers.

1)    Whenever an interface pointer is stored in an interface pointer variable, *AddRef* must be called through this interface pointer.

2)    Immediately before an interface pointer variable is cleared, overwritten, or destroyed, *Release* must be called on the interface pointer presently in the variable.

Rule 1 implies that a function that returns an interface pointer must call *AddRef* on this interface, as it stores a new interface pointer in a register or in memory (i.e., at a place where access is still possible). Examples of such functions are *IUnknown.Query-Interface* and *IFactory.CreateInstance*.

Rule 2 implies that *Release* must be called on old values of an interface pointer variable before assigning a new value, and on local interface pointers before leaving the scope.

The example below demonstrates the use of these two rules:

    IFactory* pFactory;

    HRESULT hr = GetFactory(IID_IFactory, (void**)&pFactory);

    if (SUCCEEDED(hr))    // Reference count of factory has been incremented by GetFactory

    {

        IAnimal* pAnimal1;

        hr = pFactory->CreateInstance(0, IID_IAnimal, (void**)&pAnimal1);  //     (2)

        if (SUCCEEDED(hr))    // Reference count of pAnimal1 incremented by CreateInstance

        {

            IAnimal* pAnimal2 = pAnimal1;

            pAnimal2->AddRef();     // Increment Reference Count    (1)

            pAnimal2->Sleep();     // Do something through pAnimal2



            pAnimal1->Release();     // Release Interface before assigning a new animal    (1)   (3)

            hr = factory->CreateInstance(0, IID_IAnimal, (void**)&pAnimal1);

            if (SUCCEEDED(hr))    // Reference count of pAnimal1 incremented by CreateInstance

            {

                pAnimal1->Eat();    // Do something with second animal

                pAnimal1->Release();    // scope in which pAnimal1 is declared will be closed soon

            }

            pAnimal2->Release();    // scope in which pAnimal2 is declared will be closed

        }

        pFactory->Release();    // scope in which pFactory is declared wil be closed soon

    }

In principle, every assignment to an interface pointer must be accompanied by a *Release* call (except for NULL or uninitialized interface pointers) and an *AddRef* call. However, in some special situations, *AddRef* and *Release* pairs can be omitted safely.

Unfortunately, the COM specification defines some special rules for interface parameters of methods. An interface may specify that some arguments of its methods are passed only in one direction; i.e., only from the client to the server or vice versa. This minimizes communication overhead for calls across process or machine boundaries (remote procedure calls), but it complicates the rules for the COM programmer.

