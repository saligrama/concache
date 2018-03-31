package main

import "testing"

func put (m HashMap, val int32) {
  for j := 1; j <= 100; j++ {
    go m.Put(int32(j), val)
  }
}

func get (m HashMap) {
  for j := 1; j <= 100; j++ {
    go m.Get(int32(j))
  }
}

func TestPut (t *testing.T) {
  m := New(8)
  for i := 1; i <= 8; i++ {
    go put(*m, int32(i))
  }
}

func TestGet (t *testing.T) {
  m := New(8)
  for i := 1; i <= 8; i++ {
    go get(*m)
  }
}
