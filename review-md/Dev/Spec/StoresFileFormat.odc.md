**Stores File Format**

Stream = { Byte | Structure }.

Structure = Nil | Link | Object.

Nil = nil Comment Next.

Link = (link | newlink) ObjectId Comment Next.

    (* newlink and link have same meaning in new format, but newlink causes assertion trap in old *)

    (* if Next = 0 & Comment = 0 indecates end of link chain, whereas Next = 0 & Comment = 1 indicates

        next store at Pos(Next) + 4 *)

Object = (elem | store) TypePath Comment Next Down Length Stream.

    (* elem = Models.Model-derived type; for backward comp only *)



Comment = 00000000. (* extension hook *)

ObjectId = 32bits. (* index into object dictionary *)

Next = 32bits. (* offset to next object on same nesting level; 00000000 if none *)

Down = 32bits. (* offset to first object on next lower nesting level; 00000000 if none *)

Length = 32bits. (* length of this object (including all subobjects) *)



TypePath = OldType | NewType. (* sequence of type names from most precise to root base type *)

OldType = oldType TypeId.

NewType = { NewExtension } (NewBase | OldType).

NewExtension = newExt TypeName.

NewBase = newBase TypeName.

TypeName = String.

TypeId = 32bits. (* index into type dictionary *)





nil = 80. (* nil store *)

link = 81. (* link to another elem in same file *)

store = 82. (* general store *)

elem = 83. (* elem store *)

newlink = 84. (* link to another non-elem store in same file *)

