package main

import ("fmt")

func main () {
  m := New(5)
  m.Put(1, 2)
  fmt.Println(m.Get(1))
}
