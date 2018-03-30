package main

const AVG_PER_BIN_THRESH int = 4

type Entry struct {
  key int32
  val int32
}

type HashMap struct {
  nbuckets int
  size int
  mp [][]Entry
}

func index (key int32, mod int) int {
  return int(hash(key)) % mod
}

func hash (key int32) int {
  return int(key)
}

func (m *HashMap) resize () {
  ret := New(m.nbuckets * 2)

  for i := range m.mp {
    for j := range m.mp[i] {
      ret.Put(m.mp[i][j].key, m.mp[i][j].val)
    }
  }

  m.nbuckets = ret.nbuckets
  m.mp = ret.mp
}

func New(size int) (*HashMap) {
  ret := new(HashMap)
  ret.nbuckets = size
  ret.size = 0
  ret.mp = make([][]Entry, size)
  for i := range ret.mp {
    ret.mp[i] = make([]Entry, 0)
  }
  return ret
}

func (m *HashMap) Get (key int32) (int32, bool) {
  ndx := index(key, m.nbuckets)
  bin := m.mp[ndx]
  for _, entry := range bin {
    if entry.key == key {
      return entry.val, true
    }
  }
  return 0, false
}

func (m *HashMap) Put (key int32, value int32) bool {
  if m.size/m.nbuckets > 0 {
    m.resize()
  }

  ndx := index(key, m.nbuckets)
  bin := m.mp[ndx]

  for i := range bin {
    entry := &bin[i]
    if entry.key == key {
      entry.val = value
      return true
    }
  }

  if m.size == m.nbuckets {
    return false
  }

  entry := Entry{key: key, val: value}
  bin = append(bin, entry)
  m.mp[ndx] = bin
  m.size++

  return true
}
