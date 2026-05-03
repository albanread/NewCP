**DevHeapSpy**

DEFINITION DevHeapSpy;

    VAR

        par-: RECORD

            allocated-, clusters-, heapsize-: INTEGER

        END;

    PROCEDURE GetAnchor (adr: INTEGER; OUT anchor: ARRAY OF CHAR);

    PROCEDURE ShowAnchor (adr: INTEGER);

    PROCEDURE ShowHeap;

END DevHeapSpy.

*DevHeapSpy* is a tool that visualizes the contents of heap memory. The dynamically allocated memory blocks are shown in an "interactive memory map" that is updated periodically. *DevHeapSpy* displays symbolic information about every memory block you point to with the mouse. This allows you to inspect the values of record fields in objects and browse through complex data structures.

The BlackBox heap is partitioned into *clusters*. Clusters are contiguous blocks of memory. Each cluster contains a number of heap objects. *DevHeapSpy* visualizes clusters with large grey blocks. The heap objects within a cluster are visualized by red and blue areas. The red areas represent portions of memory allocated to variables of some record type. The blue areas represent portions of memory allocated to dynamic arrays.

To display heap information, choose *Heap Spy...* from menu *Info*. This openes a small dialog box showing summary information about the heap.

To open a *DevHeapSpy* window, click on button *Show Heap* in the dialog box opened with *Heap Spy...*. This opens a window similar to the one shown in Figure 1. When you press the left mouse button within the area of a cluster, *DevHeapSpy* gives information about the object the mouse points to. If the mouse points to a heap object, i.e., to a red or blue area, the object is highlighted. If you release the mouse button while a heap object is highlighted, a debugger window will be opened that shows detailed symbolic information about the object.

The following command can be used:

    "Heap Spy..."    ""    "StdCmds.OpenAuxDialog('Dev/Rsrc/HeapSpy', 'Heap Spy')"    ""

VAR **par-**: RECORD

Interactor for the heap spy dialog.

**allocated-**: INTEGER

The number of currently allocated bytes.

**clusters-**: INTEGER

The number of currently allocated clusters.

**heapsize-**: INTEGER

The number of currently used bytes (the number of clusters times the size of one cluster).

PROCEDURE **GetAnchor** (adr: INTEGER; OUT anchor: ARRAY OF CHAR)

PROCEDURE **ShowAnchor** (adr: INTEGER)

PROCEDURE **ShowHeap**

Various procedures used in the heap spy dialog.

