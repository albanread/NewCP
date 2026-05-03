**Oberon by Example: ObxPi**

This example demonstrates the use of module *Integers* for computing with arbitrary precision decimals. The example implements the command *ObxPi.WritePi*, which computes n decimal digits of the constant Pi and writes the result into the log. The command *ObxPi.Pi *only computes Pi, without printing the result. It can be used to measure the efficiency of the arbitrary precision integer package.

For computation, the rule

     Pi = 16 * atan(1/5) - 4 * atan(1/239)

is used where atan is approximated with its Taylor series expansion at x = 0:

        atan(x) = x - x^3/3 + x^5/5 - x^7/7 + ...

The computation is performed with integers only. Each decimal x is represented as integer x * 10^d where d is the number of decimal digits to the right of the decimal point. Arithmetic operations on decimals are performed using integer arithmetic. This way, the results are chopped, not rounded. For each operation, an error of at most one ulp may be introduced. Therefore, the computation of Pi is performed with some guard digits. The final result is chopped to the desired number of decimal digits. If you want to compute more digits of Pi you have to increase the number of guard digits. Use Ceiling(Log10(1.43*n)) guard digits in order to compute n digits of Pi.

Example:

 "ObxPi.WritePi(1000)"

31415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679821480865132823066470938446095505822317253594081284811174502841027019385211055596446229489549303819644288109756659334461284756482337867831652712019091456485669234603486104543266482133936072602491412737245870066063155881748815209209628292540917153643678925903600113305305488204665213841469519415116094330572703657595919530921861173819326117931051185480744623799627495673518857527248912279381830119491298336733624406566430860213949463952247371907021798609437027705392171762931767523846748184676694051320005681271452635608277857713427577896091736371787214684409012249534301465495853710507922796892589235420199561121290219608640344181598136297747713099605187072113499999983729780499510597317328160963185950244594553469083026425223082533446850352619311881710100031378387528865875332083814206171776691473035982534904287554687311595628638823537875937519577818577805321712268066130019278766111959092164201989

[<u>ObxPi  sources</u>](../Mod/Pi.odc.md)

