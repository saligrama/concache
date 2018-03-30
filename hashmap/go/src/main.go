package main

import ("fmt")

func main () {
  m := New(5)
  m.Put(0, 0)
  fmt.Println(m.nbuckets)
  for i := range m.mp {
    for j := range m.mp[i] {
      fmt.Println(m.mp[i][j])
    }
  }
  m.Put(0, 0)
  m.Put(1, 0)
  m.Put(2, 0)
  m.Put(3, 0)
  m.Put(4, 0)
  m.Put(5, 0)
  fmt.Println(m.nbuckets)
  for i := range m.mp {
    for j := range m.mp[i] {
      fmt.Println(m.mp[i][j])
    }
  }
}
