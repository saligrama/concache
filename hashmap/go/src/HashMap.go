package main

type Entry struct {
  key int32
  val int32
}

type HashMap struct {
  fixedSize int
  size int
  mp [][]Entry
}

func (m *HashMap) index (key int32) int {
  return int(hash(key)) % m.fixedSize
}

func hash (key int32) int {
  return int(key)
}

func New(size int) (*HashMap) {
  ret := new(HashMap)
  ret.fixedSize = size
  ret.size = 0
  ret.mp = make([][]Entry, size)
  for i := range ret.mp {
    ret.mp[i] = make([]Entry, 0)
  }
  return ret
}

func (m *HashMap) Get (key int32) (int32, bool) {
  ndx := m.index(key)
  bin := m.mp[ndx]
  for _, entry := range bin {
    if entry.key == key {
      return entry.val, true
    }
  }
  return 0, false
}

func (m *HashMap) Put (key int32, value int32) bool {
  ndx := m.index(key)
  bin := m.mp[ndx]

  for i := range bin {
    entry := &bin[i]
    if entry.key == key {
      entry.val = value
      return true
    }
  }

  if m.size == m.fixedSize {
    return false
  }

  entry := Entry{key: key, val: value}
  bin = append(bin, entry)
  m.mp[ndx] = bin
  m.size++

  return true
}
