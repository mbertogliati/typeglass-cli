package main

// UserManager coordinates user operations
type UserManager struct {
service    *UserService
repository UserRepository
}

// NewUserManager creates a new UserManager
func NewUserManager(service *UserService, repo UserRepository) *UserManager {
return &UserManager{
service:    service,
repository: repo,
}
}

// LoadUser loads a user from cache or repository
func (m *UserManager) LoadUser(id string) (*User, error) {
if user, ok := m.service.GetUser(id); ok {
return user, nil
}

return m.repository.FindByID(id)
}
