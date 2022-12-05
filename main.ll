define double @sayHi(double %x, double %y) {
entry:
  %tmpadd = fadd double %x, %y
  ret double %tmpadd
}
