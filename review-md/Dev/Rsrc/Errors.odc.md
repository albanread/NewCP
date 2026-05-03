**List of Oberon Error Numbers**

Oberon microsystems  28. 2. 2003

**1. Incorrect use of language Oberon**

  0    undeclared identifier

  1    multiply defined identifier

  2    illegal character in number

  3    illegal character in string

  4    identifier does not match procedure name

  5    comment not closed

  6

  7

  8

  9    "=" expected

10

11

12    type definition starts with incorrect symbol

13    factor starts with incorrect symbol

14    statement starts with incorrect symbol

15    declaration followed by incorrect symbol

16    MODULE expected

17

18

19    "." missing

20    "," missing

21    ":" missing

22

23    ")" missing

24    "]" missing

25    "}" missing

26    OF missing

27    THEN missing

28    DO missing

29    TO missing

30

31

32

33

34

35    "," or OF expected

36    CONST, TYPE, VAR, PROCEDURE, BEGIN, or END missing

37    PROCEDURE, BEGIN, or END missing

38    BEGIN or END missing

39

40    "(" missing

41    illegally marked identifier

42    constant not an integer

43    UNTIL missing

44    ":=" missing

45

46    EXIT not within loop statement

47    string expected

48    identifier expected

49    ";" missing

50    expression should be constant

51    END missing

52    identifier does not denote a type

53    identifier does not denote a record type

54    result type of procedure is not a basic type

55    procedure call of a function

56    assignment to non-variable

57    pointer not bound to record or array type

58    recursive type definition

59    illegal open array parameter

60    wrong type of case label

61    inadmissible type of case label

62    case label defined more than once

63    illegal value of constant

64    more actual than formal parameters

65    fewer actual than formal parameters

66    element types of actual array and formal open array differ

67    actual parameter corresponding to open array is not an array

68    control variable must be integer

69    parameter must be an integer constant

70    pointer or VAR / IN record required as formal receiver

71    pointer expected as actual receiver

72    procedure must be bound to a record of the same scope

73    procedure must have level 0

74    procedure unknown in base type

75    invalid call of base procedure

76    this variable (field) is read only

77    object is not a record

78    dereferenced object is not a variable

79    indexed object is not a variable

80    index expression is not an integer

81    index out of specified bounds

82    indexed variable is not an array

83    undefined record field

84    dereferenced variable is not a pointer

85    guard or test type is not an extension of variable type

86    guard or testtype is not a pointer

87    guarded or tested variable is neither a pointer nor a VAR- or IN-parameter record

88    open array not allowed as variable, record field or array element

89    ANYRECORD may not be allocated

90    dereferenced variable is not a character array

91

92    operand of IN not an integer, or not a set

93    set element type is not an integer

94    operand of & is not of type BOOLEAN

95    operand of OR is not of type BOOLEAN

96    operand not applicable to (unary) +

97    operand not applicable to (unary) -

98    operand of ~ is not of type BOOLEAN

99    ASSERT fault

100    incompatible operands of dyadic operator

101    operand type inapplicable to *

102    operand type inapplicable to /

103    operand type inapplicable to DIV

104    operand type inapplicable to MOD

105    operand type inapplicable to +

106    operand type inapplicable to -

107    operand type inapplicable to = or #

108    operand type inapplicable to relation

109    overriding method must be exported

110    operand is not a type

111    operand inapplicable to (this) function

112    operand is not a variable

113    incompatible assignment

114    string too long to be assigned

115    parameter does not match

116    number of parameters does not match

117    result type does not match

118    export mark does not match with forward declaration

119    redefinition textually precedes procedure bound to base type

120    type of expression following IF, WHILE, UNTIL or ASSERT is not BOOLEAN

121    called object is not a procedure

122    actual VAR-, IN-, or OUT-parameter is not a variable

123    type is not identical with that of formal VAR-, IN-, or OUT-parameter

124    type of result expression differs from that of procedure

125    type of case expression is neither INTEGER nor CHAR

126    this expression cannot be a type or a procedure

127    illegal use of object

128    unsatisfied forward reference

129    unsatisfied forward procedure

130    WITH clause does not specify a variable

131    LEN not applied to array

132    dimension in LEN too large or negative

133    function without RETURN

135    SYSTEM not imported

136    LEN applied to untagged array

137    unknown array length

138    NEW not allowed for untagged structures

139    Test applied to untagged record

140    untagged receiver

141    SYSTEM.NEW not implemented

142    tagged structures not allowed for NIL compatible var parameters

143    tagged pointer not allowed in untagged structure

144    no pointers allowed in BYTES argument

145    untagged open array not allowed as value parameter

150    key inconsistency of imported module

151    incorrect symbol file

152    symbol file of imported module not found

153    object or symbol file not opened (disk full?)

154    recursive import not allowed

155    generation of new symbol file not allowed

160    interfaces must be extensions of IUnknown

161    interfaces must not have fields

162    interface procedures must be abstract

163    interface records must be abstract

164    pointer must be extension of queried interface type

165    illegal guid constant

166    AddRef & Release may not be used

167    illegal assignment to [new] parameter

168    wrong [iid] - [new] pair

169    must be an interface pointer

177    IN only allowed for records and arrays

178    illegal attribute

179    abstract methods of exported records must be exported

180    illegal receiver type

181    base type is not extensible

182    base procedure is not extensible

183    non-matching export

184    Attribute does not match with forward declaration

185    missing NEW attribute

186    illegal NEW attribute

187    new empty procedure in non extensible record

188    extensible procedure in non extensible record

189    illegal attribute change

190    record must be abstract

191    base type must be abstract

192    unimplemented abstract procedures in base types

193    abstract or limited records may not be allocated

194    no supercall allowed to abstract or empty procedures

195    empty procedures may not have out parameters or return a value

196    procedure is implement-only exported

197    extension of limited type must be limited

198    obsolete oberon type

199    obsolete oberon function

**2. Limitations of implementation**

200    not yet implemented

201    lower bound of set range greater than higher bound

202    set element greater than MAX(SET) or less than 0

203    number too large

204    product too large

205    division by zero

206    sum too large

207    difference too large

208    overflow in arithmetic shift

209    case range too large

210    code too long

211    jump distance too large

212    illegal real operation

213    too many cases in case statement

214    structure too large

215    not enough registers: simplify expression

216    not enough floating-point registers: simplify expression

217    unimplemented SYSTEM function

218    illegal value of parameter  (0 <= p < 128)

219    illegal value of parameter  (0 <= p < 16)

220    illegal value of parameter

221    too many pointers in a record

222    too many global pointers

223    too many record types

224    too many pointer types

225    illegal sys flag

226    too many exported procedures

227    too many imported modules

228    too many exported structures

229    too many nested records for import

230    too many constants (strings) in module

231    too many link table entries (external procedures)

232    too many commands in module

233    record extension hierarchy too high

235    too many modifiers

240    identifier too long

241    string too long

242    too many meta names

243    too many imported variables

249    inconsistent import

250    code proc must not be exported

251    too many nested function calls

254    debug position not found

255    debug position

260    illegal LONGINT operation

265    unsupported string operation

270    interface pointer reference counting restriction violated

**3. Warnings**

301    implicit type cast

302    guarded variable can be side-effected

303    open array (or pointer to array) containing pointers

**3.5 Analyzer Warnings**

900    never used

901    never set

902    used before set

903    set but never used

904    used as varpar, possibly not set

905    also declared in outer scope

906    access/assignment to intermediate

907    redefinition

908    new definition

909    statement after RETURN/EXIT

910    for loop variable set

911    implied type guard

912    superfluous type guard

913    call might depend on evaluation sequence of params.

930    superfluous semicolon

**4.0 Bytecode Errors**

401    bytecode restriction: no structured assignment

402    bytecode restriction: no procedure types

403    bytecode restriction: no nested procedures

404    bytecode restriction: illegal SYSTEM function

410    variable may not have been assigned

411    no proofable return

412    illegal constructor call

413    missing constructor call

