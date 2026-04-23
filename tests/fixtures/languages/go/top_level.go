package sample

import "fmt"

const (
	// State docs.
	stateReady = "ready"
	stateDone  = "done"
)

var version = buildVersion()

// User docs.
type User struct {
	ID   int
	Name string
}

// Reader docs.
type Reader interface {
	Read(p []byte) (n int, err error)
}

type ID = string

// NewUser docs.
func NewUser(name string) *User {
	fmt.Println(name)
	return &User{Name: name}
}

// NameLen docs.
func (u *User) NameLen() int {
	return len(u.Name)
}
