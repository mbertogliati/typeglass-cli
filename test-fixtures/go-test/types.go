package main

// User represents a user in the system
type User struct {
ID    string
Name  string
Email string
}

// UserService provides user management functionality
type UserService struct {
users map[string]*User
}

// NewUserService creates a new UserService
func NewUserService() *UserService {
return &UserService{
users: make(map[string]*User),
}
}

// GetUser retrieves a user by ID
func (s *UserService) GetUser(id string) (*User, bool) {
user, ok := s.users[id]
return user, ok
}

// CreateUser adds a new user
func (s *UserService) CreateUser(user *User) {
s.users[user.ID] = user
}

// UserRepository defines data access interface
type UserRepository interface {
FindByID(id string) (*User, error)
Save(user *User) error
}
