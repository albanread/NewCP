**BlackBox Object File Format**

bh    25.01.2007    procedure signatures added

bj    19.04.2001    updated file header with correct file tag and added processor field. Added semantic part of this docu.

bh     12.12.95

**Syntax:**

ObjFile    =    HeaderBlk MetaBlk DescBlk CodeBlk FixBlk UseBlk.

HeaderBlk    =    OFTag processor4 headsize4 metasize4 descsize4 codesize4 datasize4 nofimp modname {impname} Align.

OFTag    =    6FX 4FX 43X 46X.

MetaBlk    =    RefBlk Align SigBlk ExpBlk PtrBlk FldBlk ImpBlk Names Consts Align.

DescBlk    =    ModBlk Align TDescBlk.

ExpBlk    =    Directory.

Directory    =    numobj4 {Object}.

Object    =    fprint4 offs4 id4 Struct.

SigBlk    =    {Signature}.

Signature    =    Struct numPar4 {id4 Struct}.

Struct    =    form4 | strRef4 | sigRef4.

PtrBlk    =    {offset4}.

FldBlk    =    {Directory | Signature}.

ImpBlk    =    {0.4}.

Names    =    {char}.

Consts    =    {byte}.

Align    =    {0X}.

ModBlk    =    0.4 opts4 0.4 0.4 term4 nofimp4 nofptr4 csize4 dsize4 rsize4

        codeRef4 dataRef4 refsRef4 namesRef4 ptrsRef4 impRef4 expRef4 name.

TDescBlk    =    {RecDesc | ArrDesc | DArrDesc | PtrDesc | ProcDesc | FieldList}.

RecDesc    =    0.4 {methRef4} size4 modRef4 id4 {btypRef4} flistRef4 {ptrOffs4} sent4.

ArrDesc    =    nofele4 modRef4 id4 btypRef4.

PtrDesc    =    0.4 modRef4 id4 btypRef4.

ProcDesc    =    fprint4 modRef4 id4 sigRef4.

FieldList    =    Directory.

FixBlk    =    newreclink newarrlink metalink desclink codelink datalink.

link    =    {fixupadr offset} 0X.

UseBlk    =    {{UConst | UType | UVar | UProc} 0X}.

UConst    =    1X name fprint.

UType    =    2X name fprint opt link.

UVar    =    3X name fprint link.

UProc    =    4X name fprint link.

RefBlk    =    {0F8X procend name {Mode Form adr name}}.

Mode    =    Var | VarPar.

Var    =    0FDX.

VarPar    =    0FFX.

Form    =    Bool | Char | LChar | SInt | Int | LInt | Real | LReal | Set | LargeInt | AnyRec | AnyPtr | Pointer | Proc | String | Struct.

Bool    =    1X.

Char    =    2X.

LChar     =    3X.

SInt    =    4X.

Int    =    5X.

LInt    =    6X.

Real    =    7X.

LReal    =    8X.

Set    =    9X.

LargeInt    =    0AX.

AnyRec    =    0BX.

AnyPtr    =    0CX.

Pointer    =    0DX.

Proc    =    0EX.

String    =    0FX.

Struct    =    10X descRef4.

**Semantics:**

**Header**

Contains the sizes of the different blocks, the name of the module and a list of the names of the imported modules.

**Meta**

Meta information about the module. Only used by the modules *Meta* and *DevDebug*.

**Desc**

Meta information about the module which should stay in memory even when the module have been unloaded. Consists of two parts corresponding to the types Kernel.Module and Kernel.Type. The layout is made so the memory addresses can be directly mapped to variables of these types.

**Code**

Contains the code for the module. The address of the Code section is the entry point to the body of the module.

**Data**

Memory space for the variables. Variables are stored in reversed order, i.e. the last global variable starts at this address.

**Fixup**

The fixup bulk contains links into the different memory blocks where there is a need for updating the addresses. The following links exist:

*    newreclink*    link to the memory location where the procedure pointer for allocating new records is stored

*    newarrlink*    link to the memory location where the procedure pointer for allocating new arrays is stored

*    metalink*    link to the start of the fixup links in the meta bulk

*    desclink*    link to the start of the fixup links in the desc bulk

*    codelink*    link to the start of the fixup links in the code bulk

*    datalink*    link to the start of the fixup links in the data bulk

For each memory block there is a list of fixup links. A link consists of two parts:

*    fixupadr*    the start of the memory location in need of fixup

*    offset*    the offset within the memory location to the first address which needs a fixup

When *fixupadr* = 0X then the end of the list of fixups for the corresponding memory block has been reached.

At the position given by *fixupadr* + *offset* there are four bytes which should be overwritten with the new fixed *value*. But before writning to this address the four bytes should be read and interpreted as follows:

    - The most significant byte indicates the *type* of fixup that should be done.

    - The other three bytes, interpreted as an INTEGER, *x*.



The *type* and *x* are used as in the table.

Fixup Types    (mem[adr] = id * 2^24 + x, mem[adr] := value)

id    type    value    next adr

100    absolute    objadr + offs    x

101    relative    objadr + offs - adr - 4    x

102    copy    mem[objadr + offs]    x

103    table    objadr + x    adr + 4

104    table end    objadr + x    -

x = 0 indicates end of fixup for this address.

**Use**

For each imported module this section contains a list of imported constants, types, variabled and procedures. With exception for constants they all need to be fixed in the same way as other links.

