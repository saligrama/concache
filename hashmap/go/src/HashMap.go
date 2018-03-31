package main

import (
  "sync"
)

const AVG_PER_BIN_THRESH int = 4

type Entry struct {
  key int32
  val int32
}

type Bin struct {
  lock *sync.RWMutex
  entries []Entry
}

type HashMap struct {
  nbuckets int
  size int
  mp []Bin
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
    for j := range m.mp[i].entries {
      ret.Put(m.mp[i].entries[j].key, m.mp[i].entries[j].val)
    }
  }

  m.nbuckets = ret.nbuckets
  m.mp = ret.mp
}

func New(size int) (*HashMap) {
  ret := new(HashMap)
  ret.nbuckets = size
  ret.size = 0
  ret.mp = make([]Bin, size)
  for i := range ret.mp {
    ret.mp[i] = Bin{lock: &sync.RWMutex{}, entries: make([]Entry, 0)}
  }
  return ret
}

func (m *HashMap) Get (key int32) (int32, bool) {
  ndx := index(key, m.nbuckets)
  bin := m.mp[ndx]
  bin.lock.RLock()
  defer bin.lock.RUnlock()
  for _, entry := range bin.entries {
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

  bin.lock.Lock()
  defer bin.lock.Unlock()
  for i := range bin.entries {
    entry := &bin.entries[i]
    if entry.key == key {
      entry.val = value
      return true
    }
  }

  if m.size == m.nbuckets {
    return false
  }

  entry := Entry{key: key, val: value}
  bin.entries = append(bin.entries, entry)
  m.mp[ndx] = bin
  m.size++

  return true
}
